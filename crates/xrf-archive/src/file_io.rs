use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_equal, format_path};

use crate::archive_file_descriptor::ArchiveFileDescriptor;

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
  let mut source: File = File::open(&descriptor.source)?;
  let volume_size: u64 = source.metadata()?.len();
  let end: u64 = u64::from(descriptor.offset) + u64::from(descriptor.size_compressed);

  // A descriptor pointing past its volume is corrupt; failing here bounds the allocation below by what the volume can
  // actually hold.
  if end > volume_size {
    return Err(XrfError::new_read_error(format!(
      "entry '{}' declares bytes {}..{end}, beyond its volume's {volume_size}-byte end",
      descriptor.name, descriptor.offset
    )));
  }

  source.seek(SeekFrom::Start(u64::from(descriptor.offset)))?;

  let mut raw: Vec<u8> = allocate_declared(descriptor.size_compressed as usize, "an archived entry")?;

  source.read_exact(raw.as_mut_slice())?;

  // Equal sizes are how the format says "stored", so there is nothing to decompress.
  if descriptor.size_real == descriptor.size_compressed {
    return Ok(raw);
  }

  decompress_descriptor(&raw, descriptor)
}

/// Decompress an entry's payload and verify it against the checksum the archive recorded.
///
/// The decoder is bounds checked and writes into a buffer sized from the descriptor, so a corrupt entry
/// is an error rather than a read past the end of it.
pub(crate) fn decompress_descriptor(raw: &[u8], descriptor: &ArchiveFileDescriptor) -> XrfResult<Vec<u8>> {
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
