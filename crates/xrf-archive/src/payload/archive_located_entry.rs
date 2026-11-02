use std::cmp::min;
use std::fs::File;
use std::io::Write;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, new_declared_vec, read_exact_at};

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
    let stored: Vec<u8> = self.read_stored()?;

    if self.descriptor.is_compressed() {
      self.decompress(&stored)
    } else {
      Ok(stored)
    }
  }

  /// Copies one entry into an already opened target, decompressing when it is stored compressed.
  ///
  /// # Errors
  ///
  /// Returns an error when reading, allocating, writing, or resizing fails, or a compressed payload fails
  /// decompression, size validation, or checksum validation.
  pub(super) fn write_contents(&self, target: &mut File) -> XrfResult {
    if self.descriptor.is_compressed() {
      target.write_all(&self.decompress(&self.read_stored()?)?)?;
    } else {
      self.copy_stored(target)?;
    }

    target.set_len(u64::from(self.descriptor.size_real))?;

    Ok(())
  }

  /// The payload's bytes as the volume stores them.
  fn read_stored(&self) -> XrfResult<Vec<u8>> {
    let mut stored: Vec<u8> = new_declared_vec(self.descriptor.size_compressed as usize, "an archived entry")?;

    read_exact_at(self.file, stored.as_mut_slice(), self.offset)?;

    Ok(stored)
  }

  /// Copies an uncompressed payload through a fixed buffer, since a stored entry can be arbitrarily large.
  fn copy_stored(&self, target: &mut File) -> XrfResult {
    let mut remaining: usize = self.descriptor.size_real as usize;
    let mut buffer: Vec<u8> = vec![0u8; min(Self::COPY_BUFFER_SIZE, remaining.max(1))];
    let mut position: u64 = self.offset;

    while remaining > 0 {
      let block: usize = min(buffer.len(), remaining);

      read_exact_at(self.file, &mut buffer[..block], position)?;

      // `write_all`, not `write`: a short write would otherwise drop the tail of this block silently, and the caller's
      // `set_len` would pad the file to the right length, hiding the corruption.
      target.write_all(&buffer[..block])?;

      position += block as u64;
      remaining -= block;
    }

    Ok(())
  }

  /// Decompresses a stored payload and verifies it against the size and checksum the volume recorded.
  fn decompress(&self, stored: &[u8]) -> XrfResult<Vec<u8>> {
    let descriptor: &ArchiveFileDescriptor = self.descriptor;
    let mut decompressed: Vec<u8> = new_declared_vec(descriptor.size_real as usize, "a decompressed archive entry")?;

    let volume = format_path(&self.volume.path);

    let written: usize = lzokay::decompress::decompress(stored, &mut decompressed).map_err(|error| {
      XrfError::new_read_error(format!(
        "Failed to decompress '{}' from '{volume}': {error}.",
        descriptor.name
      ))
    })?;

    if written != decompressed.len() {
      return Err(XrfError::new_read_error(format!(
        "'{}' from '{volume}' decompressed to {written} bytes, and its descriptor records {}.",
        descriptor.name, descriptor.size_real
      )));
    }

    let crc: u32 = crc32fast::hash(&decompressed);

    if crc != descriptor.crc {
      return Err(XrfError::new_read_error(format!(
        "'{}' from '{volume}' decompressed with CRC {crc:#010x}, and its descriptor records {:#010x}.",
        descriptor.name, descriptor.crc
      )));
    }

    Ok(decompressed)
  }
}
