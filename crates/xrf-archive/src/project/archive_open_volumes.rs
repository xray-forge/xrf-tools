//! The project's volumes, held open for a run of reads.

use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_equal, format_path, new_declared_vec, read_exact_at};

use crate::archive_descriptor::ArchiveDescriptor;
use crate::archive_file_descriptor::ArchiveFileDescriptor;
use crate::project::ArchiveProject;

/// Bytes held in memory at a time while copying a stored entry out.
///
/// Measured against a 4.48GB set on one worker: 64KB costs 6% and 1MB costs 9%, both against this. With a pool the
/// time is flat because the copy is bound by the disk, but the buffer is per worker, so a larger one is paid once per
/// core for nothing - 1MB adds 4.4MB of resident set across 32 of them.
const COPY_BUFFER_SIZE: usize = 256 * 1024;

impl ArchiveProject {
  /// Opens every volume of the set, once, for as long as the returned value lives.
  ///
  /// Reading an entry needs the volume its payload sits in, and opening that volume per entry costs an open, an
  /// `fstat` and a seek for each of the tens of thousands of files a set holds. Callers that read more than one entry
  /// hold this instead; [`Self::read_file_bytes`] opens one for its single read, which is the same code paying the
  /// open it actually needs.
  ///
  /// # Errors
  ///
  /// Returns an IO error when a volume cannot be opened or its length cannot be read.
  pub fn open_volumes(&self) -> XrfResult<ArchiveOpenVolumes<'_>> {
    let mut files: Vec<OpenVolume> = Vec::with_capacity(self.archives.len());

    for archive in &self.archives {
      let file: File = File::open(&archive.path)?;
      let size: u64 = file.metadata()?.len();

      files.push(OpenVolume { file, size });
    }

    Ok(ArchiveOpenVolumes { project: self, files })
  }
}

/// Every volume of one project, open, positioned to serve any entry of it.
///
/// An entry names its volume by position, so serving one is an index rather than a search, and an entry the project
/// does not hold cannot be expressed. The handles are read positionally, so this is shared by reference across an
/// unpack's workers without a lock and without a cursor for them to race over.
pub struct ArchiveOpenVolumes<'a> {
  project: &'a ArchiveProject,
  /// One open handle per volume, positionally matching [`ArchiveProject::archives`].
  files: Vec<OpenVolume>,
}

/// One open volume, with the length every entry of it is bounded by.
///
/// The length is taken from the handle rather than from the volume's descriptor, so the bound belongs to the file
/// actually being read rather than to what it measured when the set was indexed.
struct OpenVolume {
  file: File,
  size: u64,
}

/// One entry proven to fit the volume it names: the handle to read it from, where it starts, and the volume itself so
/// a failure can say which file it came out of.
struct LocatedEntry<'a> {
  file: &'a File,
  offset: u64,
  volume: &'a ArchiveDescriptor,
}

impl ArchiveOpenVolumes<'_> {
  /// The project these volumes belong to.
  pub fn get_project(&self) -> &ArchiveProject {
    self.project
  }

  /// Where an entry unpacks to, relative to a destination root, from its volume's `entry_point`.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry belongs to some other project.
  pub fn get_unpack_root_of(&self, descriptor: &ArchiveFileDescriptor) -> XrfResult<&Path> {
    Ok(&self.project.get_volume_of(descriptor)?.output_root_path)
  }

  /// Reads one entry into memory, decompressing it when it is stored compressed.
  ///
  /// The caller holds the whole entry. A caller that cannot afford to should use [`Self::write_contents`], which
  /// streams a stored entry straight through instead.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry declares bytes past its volume's end, its payload cannot be decompressed, or
  /// it fails the checksum its volume recorded.
  pub fn read_bytes(&self, descriptor: &ArchiveFileDescriptor) -> XrfResult<Vec<u8>> {
    let entry: LocatedEntry<'_> = self.locate(descriptor)?;
    let mut raw: Vec<u8> = new_declared_vec(descriptor.size_compressed as usize, "an archived entry")?;

    read_exact_at(entry.file, raw.as_mut_slice(), entry.offset)?;

    // Equal sizes are how the format says "stored", so there is nothing to decompress.
    if descriptor.size_real == descriptor.size_compressed {
      return Ok(raw);
    }

    Self::decompress(&raw, descriptor, &entry.volume.path)
  }

  /// Copies one entry into an already opened target, decompressing when it is stored compressed.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry declares bytes past its volume's end, and an IO error when the copy fails.
  pub fn write_contents(&self, target: &mut File, descriptor: &ArchiveFileDescriptor) -> XrfResult {
    let entry: LocatedEntry<'_> = self.locate(descriptor)?;

    if descriptor.size_real != descriptor.size_compressed {
      let mut raw: Vec<u8> = new_declared_vec(descriptor.size_compressed as usize, "an archived entry")?;

      read_exact_at(entry.file, raw.as_mut_slice(), entry.offset)?;
      target.write_all(&Self::decompress(&raw, descriptor, &entry.volume.path)?)?;
    } else {
      // A stored entry can be arbitrarily large, so it goes through a fixed buffer rather than memory.
      let mut remaining: usize = descriptor.size_real as usize;
      let mut buffer: Vec<u8> = vec![0u8; min(COPY_BUFFER_SIZE, remaining.max(1))];
      let mut position: u64 = entry.offset;

      while remaining > 0 {
        let block: usize = min(buffer.len(), remaining);

        read_exact_at(entry.file, &mut buffer[..block], position)?;

        // `write_all`, not `write`: a short write would otherwise drop the tail of this block silently, and the
        // `set_len` below would pad the file to the right length, hiding the corruption.
        target.write_all(&buffer[..block])?;

        position += block as u64;
        remaining -= block;
      }
    }

    target.set_len(descriptor.size_real as u64)?;

    Ok(())
  }

  /// The handle holding an entry's payload and the offset it starts at, once the entry is known to fit its volume.
  ///
  /// The one place a declared extent is checked, so every read is bounded by what the volume can actually hold rather
  /// than by what a descriptor claims. Sizes read out of an archive are untrusted; this is what makes the allocation
  /// each caller performs next safe to size from one.
  fn locate(&self, descriptor: &ArchiveFileDescriptor) -> XrfResult<LocatedEntry<'_>> {
    let volume: &ArchiveDescriptor = self.project.get_volume_of(descriptor)?;
    let open: &OpenVolume = &self.files[descriptor.volume as usize];
    let offset: u64 = u64::from(descriptor.offset);
    let end: u64 = offset + u64::from(descriptor.size_compressed);

    if end > open.size {
      return Err(XrfError::new_read_error(format!(
        "entry '{}' declares bytes {}..{end}, beyond its volume's {}-byte end",
        descriptor.name, descriptor.offset, open.size
      )));
    }

    Ok(LocatedEntry {
      file: &open.file,
      offset,
      volume,
    })
  }

  /// Decompress an entry's payload and verify it against the checksum the archive recorded.
  ///
  /// The decoder is bounds checked and writes into a buffer sized from the descriptor, so a corrupt entry
  /// is an error rather than a read past the end of it.
  fn decompress(raw: &[u8], descriptor: &ArchiveFileDescriptor, volume: &Path) -> XrfResult<Vec<u8>> {
    let mut decompressed: Vec<u8> = new_declared_vec(descriptor.size_real as usize, "a decompressed archive entry")?;

    let written: usize = lzokay::decompress::decompress(raw, &mut decompressed).map_err(|error| {
      XrfError::new_read_error(format!(
        "Failed to decompress '{}' from '{}': {error}.",
        descriptor.name,
        format_path(volume)
      ))
    })?;

    assert_equal(
      written,
      decompressed.len(),
      "Decompressed size must match the descriptor",
    )?;

    assert_equal(
      descriptor.crc,
      crc32fast::hash(decompressed.as_slice()),
      "CRCs do not match",
    )?;

    Ok(decompressed)
  }
}
