use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, open_export_file};

use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::ogf::ogf_file::OgfFile;
use crate::ogf::ogf_raw_patch::OgfRawPatch;
use crate::ogf::ogf_refs_patch_report::OgfRefsPatchReport;
use crate::ogf::ogf_residue::{OgfResidue, normalize_ogf_bytes};

/// Editing operations over the motion refs of an ogf file.
pub struct OgfMotionRefsProcessor {}

impl OgfMotionRefsProcessor {
  /// Rewrite motion refs of an ogf file, verifying the result before letting it survive.
  ///
  /// Guards, in order: rewriting the file's existing refs must reproduce the source bytes exactly,
  /// the patched file must read back the requested refs, and a failure of either leaves neither a
  /// corrupted source nor a partial destination behind.
  pub fn patch_motion_refs_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    motion_refs: &[String],
    is_dry_run: bool,
  ) -> XrfResult<OgfRefsPatchReport> {
    let original: Vec<u8> = fs::read(source)?;
    let existing: Vec<String> = OgfFile::read_motion_refs_from_path::<T, _>(&source)?;

    Self::assert_chunk_copy_is_lossless::<T>(source, &original, &existing)?;

    let patched: Vec<u8> = Self::write_motion_refs_to_buffer::<T>(OgfRawPatch::open_source(source)?, motion_refs)?;

    let report: OgfRefsPatchReport = OgfRefsPatchReport {
      original_size: original.len(),
      patched_size: patched.len(),
      // The buffer writer refuses anything other than exactly one refs chunk.
      patched_count: 1,
      discarded_size: original
        .len()
        .saturating_sub(normalize_ogf_bytes::<T>(&original)?.len()),
      is_dry_run,
    };

    if is_dry_run {
      return Ok(report);
    }

    if let Some(parent) = destination.parent() {
      fs::create_dir_all(parent)?;
    }

    open_export_file(destination)?.write_all(&patched)?;

    if let Err(error) = Self::assert_written_refs_match::<T>(destination, motion_refs) {
      OgfRawPatch::revert_destination(source, destination, &original)?;

      return Err(error);
    }

    Ok(report)
  }

  /// Rewrite motion refs of an ogf file, copying every other chunk verbatim.
  ///
  /// The result is well formed even when the source was not: bytes the engine's loader never reads are discarded
  /// rather than carried over. See [`crate::normalize_ogf_bytes`] for what that covers and why the discarded path is
  /// reported before it goes.
  pub fn write_motion_refs_to_buffer<T: ByteOrder>(file: File, motion_refs: &[String]) -> XrfResult<Vec<u8>> {
    let (mut chunks, _) =
      OgfResidue::read_root_chunks::<T, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_file(file)?)?;
    let mut buffer: Vec<u8> = Vec::new();
    let mut patched_count: u32 = 0;

    for chunk in &mut chunks {
      let payload: Vec<u8> = if OgfRawPatch::is_kinematics_chunk(chunk.id) {
        patched_count += 1;

        let mut kinematics_writer: ChunkWriter = ChunkWriter::new();

        OgfKinematicsChunk {
          source_chunk_id: chunk.id,
          motion_refs: motion_refs.to_vec(),
          // Dropped, not carried: a patched file is well formed, and the engine loads it identically.
          trailing: Vec::new(),
        }
        .write::<T>(&mut kinematics_writer)?;

        kinematics_writer.flush_raw_into_buffer()?
      } else {
        chunk.reset_pos()?;
        chunk.read_remaining()?
      };

      let mut chunk_writer: ChunkWriter = ChunkWriter::new();

      chunk_writer.write_all(&payload)?;
      chunk_writer.flush_chunk_into::<T>(&mut buffer, chunk.id)?;
    }

    if patched_count != 1 {
      return Err(XrfError::new_invalid_error(format!(
        "Expected exactly one ogf motion refs chunk to rewrite, got {patched_count}"
      )));
    }

    Ok(buffer)
  }

  /// Guard that rewriting the refs a file already has reproduces that file, normalized.
  fn assert_chunk_copy_is_lossless<T: ByteOrder>(source: &Path, original: &[u8], existing: &[String]) -> XrfResult {
    let reverted: Vec<u8> = Self::write_motion_refs_to_buffer::<T>(OgfRawPatch::open_source(source)?, existing)?;

    OgfRawPatch::assert_chunk_copy_is_lossless::<T>(source, original, &reverted, "rewriting its existing motion refs")
  }

  /// Guard that the written file reads back exactly the requested motion refs.
  fn assert_written_refs_match<T: ByteOrder>(destination: &Path, motion_refs: &[String]) -> XrfResult {
    let read_back: Vec<String> = OgfFile::read_motion_refs_from_path::<T, _>(&destination)?;

    if read_back != motion_refs {
      return Err(XrfError::new_verify_error(format!(
        "Patched {} reads back motion refs {:?} instead of {:?}",
        format_path(destination),
        read_back,
        motion_refs
      )));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::fs::File;
  use std::io::Write;
  use std::path::PathBuf;

  use xrf_chunk::{ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    overwrite_generated_test_resource_as_file,
  };

  use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
  use crate::ogf::ogf_file::OgfFile;
  use crate::ogf::ogf_motion_refs_processor::OgfMotionRefsProcessor;

  /// Payload standing in for a chunk the writer must copy verbatim, such as geometry.
  const OPAQUE_PAYLOAD: [u8; 12] = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 255, 128];

  fn write_sample(filename: &str, refs_chunk_id: u32, motion_refs: &[String]) -> XrfResult<PathBuf> {
    let mut opaque_writer: ChunkWriter = ChunkWriter::new();
    opaque_writer.write_all(&OPAQUE_PAYLOAD)?;

    let mut refs_writer: ChunkWriter = ChunkWriter::new();
    OgfKinematicsChunk {
      source_chunk_id: refs_chunk_id,
      motion_refs: motion_refs.to_vec(),
      trailing: Vec::new(),
    }
    .write::<XRayByteOrder>(&mut refs_writer)?;

    let mut file: File = overwrite_generated_test_resource_as_file(filename)?;

    opaque_writer.flush_chunk_into::<XRayByteOrder>(&mut file, 9)?;
    refs_writer.flush_chunk_into::<XRayByteOrder>(&mut file, refs_chunk_id)?;

    Ok(build_absolute_generated_test_resource_path(filename))
  }

  #[test]
  fn test_write_motion_refs_reproduces_source_when_unchanged() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "unchanged.ogf");
    let refs: Vec<String> = vec![String::from("dynamics\\weapons\\wpn_ak74\\anim")];
    let path: PathBuf = write_sample(&filename, OgfKinematicsChunk::CHUNK_ID, &refs)?;

    let rewritten: Vec<u8> =
      OgfMotionRefsProcessor::write_motion_refs_to_buffer::<XRayByteOrder>(File::open(&path)?, &refs)?;

    assert_eq!(
      rewritten,
      fs::read(&path)?,
      "Expect rewriting existing refs to reproduce source bytes"
    );

    Ok(())
  }

  #[test]
  fn test_write_motion_refs_preserves_other_chunks() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "preserves_chunks.ogf");
    let path: PathBuf = write_sample(&filename, OgfKinematicsChunk::CHUNK_ID, &[String::from("old\\ref")])?;

    let patched: Vec<u8> = OgfMotionRefsProcessor::write_motion_refs_to_buffer::<XRayByteOrder>(
      File::open(&path)?,
      &[String::from("new\\much\\longer\\ref"), String::from("second")],
    )?;

    // Opaque chunk header and payload must survive untouched at the head of the file.
    let original: Vec<u8> = fs::read(&path)?;
    let opaque_len: usize = 8 + OPAQUE_PAYLOAD.len();

    assert_eq!(
      patched[..opaque_len],
      original[..opaque_len],
      "Expect leading opaque chunk to be copied verbatim"
    );

    fs::write(&path, &patched)?;

    assert_eq!(
      OgfFile::read_motion_refs_from_path::<XRayByteOrder, _>(&path)?,
      vec![String::from("new\\much\\longer\\ref"), String::from("second")],
      "Expect patched file to read back new refs"
    );

    Ok(())
  }

  #[test]
  fn test_write_motion_refs_preserves_old_chunk_id() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "old_chunk_id.ogf");
    let path: PathBuf = write_sample(&filename, OgfKinematicsChunk::CHUNK_ID_OLD, &[String::from("old\\ref")])?;

    let patched: Vec<u8> = OgfMotionRefsProcessor::write_motion_refs_to_buffer::<XRayByteOrder>(
      File::open(&path)?,
      &[String::from("replacement")],
    )?;

    fs::write(&path, &patched)?;

    let chunk_id: u32 = u32::from_le_bytes(
      patched[patched.len() - 8 - "replacement".len() - 1..][..4]
        .try_into()
        .expect("Chunk id bytes"),
    );

    assert_eq!(
      chunk_id,
      OgfKinematicsChunk::CHUNK_ID_OLD,
      "Expect source chunk id form to be preserved"
    );

    assert_eq!(
      OgfFile::read_motion_refs_from_path::<XRayByteOrder, _>(&path)?,
      vec![String::from("replacement")]
    );

    Ok(())
  }

  #[test]
  fn test_write_motion_refs_requires_refs_chunk() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "without_refs.ogf");
    let mut opaque_writer: ChunkWriter = ChunkWriter::new();

    opaque_writer.write_all(&OPAQUE_PAYLOAD)?;
    opaque_writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 9)?;

    let path: PathBuf = build_absolute_generated_test_resource_path(&filename);

    assert!(
      OgfMotionRefsProcessor::write_motion_refs_to_buffer::<XRayByteOrder>(File::open(&path)?, &[String::from("any")])
        .is_err(),
      "Expect rewrite to be refused when file has no motion refs chunk"
    );

    Ok(())
  }
}
