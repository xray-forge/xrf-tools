use std::fs;
use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::residue::OgfNormalization;

/// Machinery both raw ogf patchers need, held once so they cannot drift apart.
pub(crate) struct OgfRawPatch {}

impl OgfRawPatch {
  pub(crate) fn open_source(source: &Path) -> XrfResult<File> {
    File::open(source).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "OGF file was not read: {}, error: {}",
        format_path(source),
        error
      ))
    })
  }

  /// Guard that a rewrite which changes nothing reproduces the source, normalized.
  pub(crate) fn assert_chunk_copy_is_lossless<T: ByteOrder>(
    source: &Path,
    original: &[u8],
    reverted: &[u8],
    identity: &str,
  ) -> XrfResult {
    let expected: Vec<u8> = OgfNormalization::normalize_bytes::<T>(original)?;

    if reverted != expected {
      return Err(XrfError::new_verify_error(format!(
        "Refused to patch {}, {} did not reproduce the source file, {} bytes original, {} bytes expected and {} bytes rewritten",
        format_path(source),
        identity,
        original.len(),
        expected.len(),
        reverted.len()
      )));
    }

    Ok(())
  }

  /// Undo a failed write, leaving neither a corrupted source nor a partial destination behind.
  pub(crate) fn revert_destination(source: &Path, destination: &Path, original: &[u8]) -> XrfResult {
    if destination == source {
      fs::write(destination, original)?;
    } else if destination.exists() {
      fs::remove_file(destination)?;
    }

    Ok(())
  }

  /// Payload of a motion refs chunk with the bytes the count never reaches removed.
  pub(crate) fn read_normalized_kinematics_payload<T: ByteOrder, D: ChunkDataSource>(
    chunk: &ChunkReader<D>,
  ) -> XrfResult<Vec<u8>> {
    let mut kinematics: OgfKinematicsChunk = OgfKinematicsChunk::read::<T, _>(&mut chunk.clone(), chunk.id)?;
    let mut writer: ChunkWriter = ChunkWriter::new();

    kinematics.trailing.clear();
    kinematics.write::<T>(&mut writer)?;

    writer.flush_raw_into_buffer()
  }

  pub(crate) fn is_kinematics_chunk(id: u32) -> bool {
    id == OgfKinematicsChunk::CHUNK_ID || id == OgfKinematicsChunk::CHUNK_ID_OLD
  }
}
