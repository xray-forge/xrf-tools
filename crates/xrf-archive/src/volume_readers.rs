//! Reading entry payloads out of a volume set that is opened once rather than once per entry.

use std::cmp::min;
use std::fs::File;
use std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};
use std::path::Path;
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::archive_file_descriptor::ArchiveFileDescriptor;
use crate::file_io::{allocate_declared, decompress_descriptor};
use crate::project::ArchiveProject;

/// Bytes held in memory at a time while copying a stored entry out.
const COPY_BUFFER_SIZE: usize = 256 * 1024;

/// Reads exactly `buffer.len()` bytes from `offset` without moving the file's own cursor.
///
/// Positional rather than seek-then-read because the handle is shared: an unpack runs one worker per core over entries
/// dispatched in hash order, and a cursor is state two of them would race for. Both platform calls take `&File`, so a
/// volume needs neither a lock nor a handle per worker.
#[cfg(unix)]
fn read_exact_at(file: &File, buffer: &mut [u8], offset: u64) -> IoResult<()> {
  use std::os::unix::fs::FileExt;

  file.read_exact_at(buffer, offset)
}

/// Windows offers only the short-read form, so the loop that Unix's `read_exact_at` already provides lives here.
#[cfg(windows)]
fn read_exact_at(file: &File, buffer: &mut [u8], offset: u64) -> IoResult<()> {
  use std::os::windows::fs::FileExt;

  let mut read: usize = 0;

  while read < buffer.len() {
    match file.seek_read(&mut buffer[read..], offset + read as u64)? {
      0 => {
        return Err(IoError::new(
          ErrorKind::UnexpectedEof,
          "volume ended before the entry it declares",
        ));
      }
      count => read += count,
    }
  }

  Ok(())
}

/// One volume, open, with the length every entry of it is checked against.
struct OpenVolume {
  file: File,
  path: Arc<Path>,
  size: u64,
}

/// Every volume of a set, opened once and read by position.
///
/// Opening a volume per entry cost an open, an `fstat` and a seek for each of the tens of thousands of files a set
/// holds, which is most of what unpacking one used to spend on a small entry. The bounds check each of those `fstat`
/// calls paid for is done once per volume here instead, and kept: it is what bounds every later read and allocation
/// by what the volume can actually hold.
///
/// Shared by reference across an unpack's workers. Every read is positional, so the handles carry no cursor to race.
pub struct ArchiveVolumeReaders {
  volumes: Vec<OpenVolume>,
}

impl ArchiveVolumeReaders {
  /// Opens every volume the project's entries name, once each.
  ///
  /// Taken from the entries rather than from `archives` on purpose: what has to be open is exactly what the entries
  /// point at, and deriving it from the volume list instead would let a project whose two halves disagree fail on a
  /// read it could have served. Entries of one volume share its path allocation, so the distinct set is found by
  /// pointer in one pass.
  ///
  /// # Errors
  ///
  /// Returns an IO error when a volume cannot be opened or its length cannot be read.
  pub fn open(project: &ArchiveProject) -> XrfResult<Self> {
    let mut volumes: Vec<OpenVolume> = Vec::with_capacity(project.archives.len());

    for descriptor in project.files.values() {
      if volumes
        .iter()
        .any(|volume| Arc::ptr_eq(&volume.path, &descriptor.source) || volume.path == descriptor.source)
      {
        continue;
      }

      let file: File = File::open(&descriptor.source)?;
      let size: u64 = file.metadata()?.len();

      volumes.push(OpenVolume {
        file,
        path: Arc::clone(&descriptor.source),
        size,
      });
    }

    Ok(Self { volumes })
  }

  /// Copies one archived entry into an already opened target, decompressing when it is stored compressed.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry names a volume this set did not open or declares bytes past its end, and an
  /// IO error when the copy itself fails.
  pub fn write_descriptor_contents(&self, target: &mut File, descriptor: &ArchiveFileDescriptor) -> XrfResult {
    let volume: &OpenVolume = self.get_volume_of(descriptor)?;

    write_entry(&volume.file, volume.size, target, descriptor)
  }

  /// The open volume an entry's payload sits in.
  ///
  /// Identity first, because every entry of a volume shares that volume's path allocation, so the pointer comparison
  /// answers without touching the path. The equality fallback keeps a descriptor from some other project working
  /// rather than reporting a volume that is plainly present. A set has a handful of volumes, so the scan is shorter
  /// than hashing the path would be.
  fn get_volume_of(&self, descriptor: &ArchiveFileDescriptor) -> XrfResult<&OpenVolume> {
    self
      .volumes
      .iter()
      .find(|volume| Arc::ptr_eq(&volume.path, &descriptor.source) || volume.path == descriptor.source)
      .ok_or_else(|| {
        XrfError::new_read_error(format!(
          "entry '{}' names volume '{}', which is not one this archive opened",
          descriptor.name,
          format_path(&descriptor.source)
        ))
      })
  }
}

/// Copy one archived entry into an already opened target, decompressing when it is stored compressed.
///
/// Opens the entry's volume for this one call. Use [`ArchiveVolumeReaders`] to write more than one entry of a set:
/// this pays an open and an `fstat` every time, which is what makes it the wrong shape for a loop.
///
/// # Errors
///
/// Returns a read error when the entry declares bytes past its volume's end, and an IO error when the volume cannot be
/// opened or the copy fails.
pub fn write_descriptor_contents(target: &mut File, descriptor: &ArchiveFileDescriptor) -> XrfResult {
  let source: File = File::open(&descriptor.source)?;
  let volume_size: u64 = source.metadata()?.len();

  write_entry(&source, volume_size, target, descriptor)
}

/// The one copy path, so a single extraction and a whole unpack cannot drift on bounds checking, CRC verification, or
/// how a stored entry is streamed.
fn write_entry(source: &File, volume_size: u64, target: &mut File, descriptor: &ArchiveFileDescriptor) -> XrfResult {
  let offset: u64 = u64::from(descriptor.offset);
  let end: u64 = offset + u64::from(descriptor.size_compressed);

  // A descriptor pointing past its volume is corrupt; failing here bounds every later read and allocation by
  // what the volume can actually hold.
  if end > volume_size {
    return Err(XrfError::new_read_error(format!(
      "entry '{}' declares bytes {}..{end}, beyond its volume's {volume_size}-byte end",
      descriptor.name, descriptor.offset
    )));
  }

  if descriptor.size_real != descriptor.size_compressed {
    let mut raw: Vec<u8> = allocate_declared(descriptor.size_compressed as usize, "an archived entry")?;

    read_exact_at(source, raw.as_mut_slice(), offset)?;
    target.write_all(&decompress_descriptor(&raw, descriptor)?)?;
  } else {
    // A stored entry can be arbitrarily large, so it goes through a fixed buffer rather than memory.
    let mut remaining: usize = descriptor.size_real as usize;
    let mut buffer: Vec<u8> = vec![0u8; min(COPY_BUFFER_SIZE, remaining.max(1))];
    let mut position: u64 = offset;

    while remaining > 0 {
      let block: usize = min(buffer.len(), remaining);

      read_exact_at(source, &mut buffer[..block], position)?;

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
