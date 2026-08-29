use std::fs;
use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::ogf::ogf_residue::normalize_ogf_bytes;

/// Machinery both raw ogf patchers need, held once so they cannot drift apart.
///
/// They rewrite one chunk and copy the rest, and each has to prove the copy lost nothing, undo a failed write, and
/// discard the same engine-ignored bytes. Those are the same jobs whichever reference is being edited.
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
  ///
  /// Geometry is not re-serializable, so comparing whole files is the only proof that the chunk copy preserves
  /// everything outside the edited payload. It proves two things at once: the serializer round trips for the chunk that
  /// was rebuilt from parsed values, and framing is reproduced for every chunk copied verbatim.
  ///
  /// The target is [`normalize_ogf_bytes`] rather than the source itself because a patch drops what the engine never
  /// reads, so byte-identical output is impossible by construction for a malformed visual. For a well-formed one the
  /// two are the same bytes and this is the guard it has always been. The target is byte surgery on the source, never a
  /// second run of the writer, so a serializer bug still shows up here as a mismatch.
  pub(crate) fn assert_chunk_copy_is_lossless<T: ByteOrder>(
    source: &Path,
    original: &[u8],
    reverted: &[u8],
    identity: &str,
  ) -> XrfResult {
    let expected: Vec<u8> = normalize_ogf_bytes::<T>(original)?;

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
  ///
  /// A patcher that copies this chunk verbatim would carry those bytes into a file that is otherwise well-formed, and
  /// then disagree with [`normalize_ogf_bytes`] about what a rewrite should produce.
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
