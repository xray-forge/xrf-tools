use std::cmp::min;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert, assert_equal, assert_not_equal, format_path};

use crate::archive_file_descriptor::ArchiveFileDescriptor;

/// Bytes held in memory at a time while copying a stored entry out.
const COPY_BUFFER_SIZE: usize = 256 * 1024;

/// Allocates a buffer for a size a volume declared, failing instead of aborting when it cannot be satisfied.
///
/// Sizes read out of an archive are untrusted: a corrupt descriptor can claim gigabytes, and a plain `vec![0; n]` would
/// abort the whole process on allocation failure. `try_reserve` turns that into an error the caller reports.
pub(crate) fn allocate_declared(size: usize, what: &str) -> XrfResult<Vec<u8>> {
  let mut buffer: Vec<u8> = Vec::new();

  buffer
    .try_reserve_exact(size)
    .map_err(|error| XrfError::new_read_error(format!("cannot allocate {size} bytes for {what}: {error}")))?;
  buffer.resize(size, 0u8);

  Ok(buffer)
}

/// Read one archived entry into memory, decompressing it when it is stored compressed.
///
/// The caller holds the whole entry, so a caller that cannot afford to should use
/// [`write_descriptor_contents`] instead, which streams a stored entry straight through.
pub(crate) fn read_descriptor_bytes(descriptor: &ArchiveFileDescriptor) -> XrfResult<Vec<u8>> {
  let mut source: File = open_at_descriptor(descriptor)?;
  let mut raw: Vec<u8> = allocate_declared(descriptor.size_compressed as usize, "an archived entry")?;

  source.read_exact(raw.as_mut_slice())?;

  // Equal sizes are how the format says "stored", so there is nothing to decompress.
  if descriptor.size_real == descriptor.size_compressed {
    return Ok(raw);
  }

  decompress_descriptor(&raw, descriptor)
}

/// Copy one archived entry into an already opened target, decompressing when it is stored compressed.
///
/// Shared by whole-archive unpacking and single file extraction so the two cannot drift on CRC
/// verification or on how stored entries are streamed.
pub fn write_descriptor_contents(target: &mut File, descriptor: &ArchiveFileDescriptor) -> XrfResult {
  let mut source: File = open_at_descriptor(descriptor)?;

  if descriptor.size_real != descriptor.size_compressed {
    let mut raw: Vec<u8> = allocate_declared(descriptor.size_compressed as usize, "an archived entry")?;

    source.read_exact(raw.as_mut_slice())?;
    target.write_all(&decompress_descriptor(&raw, descriptor)?)?;
  } else {
    // A stored entry can be arbitrarily large, so it goes through a fixed buffer rather than memory.
    let mut remaining: usize = descriptor.size_real as usize;
    let mut buffer: Vec<u8> = vec![0u8; min(COPY_BUFFER_SIZE, remaining.max(1))];

    while remaining > 0 {
      let to_read: usize = min(buffer.len(), remaining);
      let read: usize = source.read(&mut buffer[..to_read])?;

      assert(read <= remaining, "Must not read more bytes than remaining")?;
      assert_not_equal(read, 0, "Unexpected End Of File")?;

      // `write_all`, not `write`: a short write would otherwise drop the tail of this block silently, and the
      // `set_len` below would pad the file to the right length, hiding the corruption.
      target.write_all(&buffer[..read])?;

      remaining -= read;
    }
  }

  target.set_len(descriptor.size_real as u64)?;

  Ok(())
}

/// Open the volume holding an entry, positioned at its payload.
fn open_at_descriptor(descriptor: &ArchiveFileDescriptor) -> XrfResult<File> {
  let mut source: File = File::open(&descriptor.source)?;
  let volume_size: u64 = source.metadata()?.len();
  let end: u64 = u64::from(descriptor.offset) + u64::from(descriptor.size_compressed);

  // A descriptor pointing past its volume is corrupt; failing here bounds every later read and allocation by
  // what the volume can actually hold.
  if end > volume_size {
    return Err(XrfError::new_read_error(format!(
      "entry '{}' declares bytes {}..{end}, beyond its volume's {volume_size}-byte end",
      descriptor.name, descriptor.offset
    )));
  }

  source.seek(SeekFrom::Start(u64::from(descriptor.offset)))?;

  Ok(source)
}

/// Decompress an entry's payload and verify it against the checksum the archive recorded.
///
/// The decoder is bounds checked and writes into a buffer sized from the descriptor, so a corrupt entry
/// is an error rather than a read past the end of it.
fn decompress_descriptor(raw: &[u8], descriptor: &ArchiveFileDescriptor) -> XrfResult<Vec<u8>> {
  let mut decompressed: Vec<u8> = allocate_declared(descriptor.size_real as usize, "a decompressed archive entry")?;

  let written: usize = lzokay::decompress::decompress(raw, &mut decompressed).map_err(|error| {
    XrfError::new_read_error(format!(
      "Failed to decompress '{}' from '{}': {error}.",
      descriptor.name,
      format_path(&descriptor.source)
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
