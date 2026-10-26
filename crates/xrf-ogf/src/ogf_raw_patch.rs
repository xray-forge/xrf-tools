use std::fs;
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
  /// The whole source, read once for every view a patch takes of it.
  pub(crate) fn read_source(source: &Path) -> XrfResult<Vec<u8>> {
    fs::read(source).map_err(|error| {
      XrfError::new_io_error(
        format!("OGF file was not read: {}, error: {error}", format_path(source)),
        error.kind(),
      )
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

  /// Payload of a motion refs chunk with the bytes the count never reaches cut off, every byte before them as stored.
  pub(crate) fn read_normalized_kinematics_payload<T: ByteOrder, D: ChunkDataSource>(
    chunk: &ChunkReader<D>,
  ) -> XrfResult<Vec<u8>> {
    let trailing: usize = OgfKinematicsChunk::read::<T, _>(&mut chunk.clone(), chunk.id)?
      .trailing
      .len();
    let mut payload: Vec<u8> = Self::read_payload(chunk)?;

    payload.truncate(payload.len() - trailing);

    Ok(payload)
  }

  /// A chunk's payload as stored, whatever its reader's cursor.
  pub(crate) fn read_payload<D: ChunkDataSource>(chunk: &ChunkReader<D>) -> XrfResult<Vec<u8>> {
    let mut chunk: ChunkReader<D> = chunk.clone();

    chunk.reset_pos()?;
    chunk.read_remaining()
  }

  /// Appends a chunk of `id` holding `payload` to `buffer`, its header declaring the payload's own size.
  pub(crate) fn write_chunk<T: ByteOrder>(buffer: &mut Vec<u8>, id: u32, payload: Vec<u8>) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.buffer = payload;
    writer.flush_chunk_into::<T>(buffer, id)?;

    Ok(())
  }

  pub(crate) fn is_kinematics_chunk(id: u32) -> bool {
    id == OgfKinematicsChunk::CHUNK_ID || id == OgfKinematicsChunk::CHUNK_ID_OLD
  }
}
