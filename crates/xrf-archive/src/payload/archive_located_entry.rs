use std::cmp::min;
use std::fs::File;
use std::io::Write;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_equal, format_path, new_declared_vec, read_exact_at};

use crate::{ArchiveDescriptor, ArchiveFileDescriptor};

/// An entry whose stored byte range has been checked against its open volume.
pub(super) struct ArchiveLocatedEntry<'a> {
  pub(super) file: &'a File,
  pub(super) offset: u64,
  pub(super) volume: &'a ArchiveDescriptor,
  pub(super) descriptor: &'a ArchiveFileDescriptor,
}

impl ArchiveLocatedEntry<'_> {
  /// Bytes held in memory at a time while copying a stored entry out.
  ///
  /// Measured against a 4.48GB set on one worker: 64KB costs 6% and 1MB costs 9%, both against this. With a pool the
  /// time is flat because the copy is bound by the disk, but the buffer is per worker, so a larger one is paid once per
  /// core for nothing - 1MB adds 4.4MB of resident set across 32 of them.
  const COPY_BUFFER_SIZE: usize = 256 * 1024;

  /// Reads one entry into memory, decompressing it when it is stored compressed.
  ///
  /// # Errors
  ///
  /// Returns an error when reading or allocating the payload fails, or a compressed payload fails decompression,
  /// size validation, or checksum validation.
  pub(super) fn read_bytes(&self) -> XrfResult<Vec<u8>> {
    let descriptor: &ArchiveFileDescriptor = self.descriptor;
    let mut raw: Vec<u8> = new_declared_vec(descriptor.size_compressed as usize, "an archived entry")?;

    read_exact_at(self.file, raw.as_mut_slice(), self.offset)?;

    // Equal sizes are how the format says "stored", so there is nothing to decompress.
    if descriptor.size_real == descriptor.size_compressed {
      return Ok(raw);
    }

    self.decompress(&raw)
  }

  /// Copies one entry into an already opened target, decompressing when it is stored compressed.
  ///
  /// # Errors
  ///
  /// Returns an error when reading, allocating, writing, or resizing fails, or a compressed payload fails
  /// decompression, size validation, or checksum validation.
  pub(super) fn write_contents(&self, target: &mut File) -> XrfResult {
    let descriptor: &ArchiveFileDescriptor = self.descriptor;

    if descriptor.size_real != descriptor.size_compressed {
      let mut raw: Vec<u8> = new_declared_vec(descriptor.size_compressed as usize, "an archived entry")?;

      read_exact_at(self.file, raw.as_mut_slice(), self.offset)?;
      target.write_all(&self.decompress(&raw)?)?;
    } else {
      // A stored entry can be arbitrarily large, so it goes through a fixed buffer rather than memory.
      let mut remaining: usize = descriptor.size_real as usize;
      let mut buffer: Vec<u8> = vec![0u8; min(Self::COPY_BUFFER_SIZE, remaining.max(1))];
      let mut position: u64 = self.offset;

      while remaining > 0 {
        let block: usize = min(buffer.len(), remaining);

        read_exact_at(self.file, &mut buffer[..block], position)?;

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

  /// Decompress an entry's payload and verify it against the checksum the archive recorded.
  fn decompress(&self, raw: &[u8]) -> XrfResult<Vec<u8>> {
    let descriptor: &ArchiveFileDescriptor = self.descriptor;
    let mut decompressed: Vec<u8> = new_declared_vec(descriptor.size_real as usize, "a decompressed archive entry")?;

    let written: usize = lzokay::decompress::decompress(raw, &mut decompressed).map_err(|error| {
      XrfError::new_read_error(format!(
        "Failed to decompress '{}' from '{}': {error}.",
        descriptor.name,
        format_path(&self.volume.path)
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
