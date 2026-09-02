use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, open_export_file};

use crate::ogf::chunks::ogf_children_chunk::OgfChildrenChunk;
use crate::ogf::chunks::ogf_texture_chunk::OgfTextureChunk;
use crate::ogf::ogf_file::OgfFile;
use crate::ogf::ogf_raw_patch::OgfRawPatch;
use crate::ogf::ogf_refs_patch_report::OgfRefsPatchReport;
use crate::ogf::residue::{OgfNormalization, OgfResidue};

/// Editing operations over the texture refs of an ogf file.
pub struct OgfTextureRefsProcessor {}

impl OgfTextureRefsProcessor {
  /// Rename a texture reference of an ogf file, verifying the result before letting it survive.
  ///
  /// Guards, in order: renaming a reference to itself must reproduce the source bytes exactly, the
  /// rename must match at least one reference so a typo cannot pass as success, the written file
  /// must name the new reference and not the old one, and a failure of any of these leaves neither
  /// a corrupted source nor a partial destination behind.
  pub fn patch_texture_refs_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    from: &str,
    to: &str,
    is_dry_run: bool,
  ) -> XrfResult<OgfRefsPatchReport> {
    let original: Vec<u8> = fs::read(source)?;
    let existing: Vec<String> = OgfFile::read_texture_refs_from_path::<T, _>(&source)?;

    Self::assert_chunk_copy_is_lossless::<T>(source, &original, from)?;

    let (patched, patched_count) =
      Self::write_texture_refs_to_buffer::<T>(OgfRawPatch::open_source(source)?, from, to)?;

    if patched_count == 0 {
      return Err(XrfError::new_verify_error(format!(
        "Refused to patch {}, no texture reference matched '{}', found {:?}",
        format_path(source),
        from,
        existing
      )));
    }

    let report: OgfRefsPatchReport = OgfRefsPatchReport {
      original_size: original.len(),
      patched_size: patched.len(),
      patched_count,
      discarded_size: original
        .len()
        .saturating_sub(OgfNormalization::normalize_bytes::<T>(&original)?.len()),
      is_dry_run,
    };

    if is_dry_run {
      return Ok(report);
    }

    if let Some(parent) = destination.parent() {
      fs::create_dir_all(parent)?;
    }

    open_export_file(destination)?.write_all(&patched)?;

    if let Err(error) = Self::assert_written_refs_match::<T>(destination, from, to) {
      OgfRawPatch::revert_destination(source, destination, &original)?;

      return Err(error);
    }

    Ok(report)
  }

  /// Rename texture references of an ogf file, copying every other chunk verbatim.
  ///
  /// The result is well formed even when the source was not, on the same terms as the motion refs patcher: bytes the
  /// engine's loader never reads are discarded rather than carried over, including those inside a motion refs chunk
  /// this command otherwise never touches.
  pub fn write_texture_refs_to_buffer<T: ByteOrder>(file: File, from: &str, to: &str) -> XrfResult<(Vec<u8>, u32)> {
    let (mut chunks, _) =
      OgfResidue::read_root_chunks::<T, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_file(file)?)?;
    let mut buffer: Vec<u8> = Vec::new();
    let mut patched_count: u32 = 0;

    for chunk in &mut chunks {
      let payload: Vec<u8> = if chunk.id == OgfChildrenChunk::CHUNK_ID {
        Self::rename_children_texture_refs::<T, _>(chunk, from, to, &mut patched_count)?
      } else if OgfRawPatch::is_kinematics_chunk(chunk.id) {
        OgfRawPatch::read_normalized_kinematics_payload::<T, _>(chunk)?
      } else {
        chunk.reset_pos()?;
        chunk.read_remaining()?
      };

      let mut chunk_writer: ChunkWriter = ChunkWriter::new();

      chunk_writer.write_all(&payload)?;
      chunk_writer.flush_chunk_into::<T>(&mut buffer, chunk.id)?;
    }

    Ok((buffer, patched_count))
  }

  /// Rebuild a children container payload, renaming texture references inside each nested object.
  fn rename_children_texture_refs<T: ByteOrder, D: ChunkDataSource>(
    chunk: &mut ChunkReader<D>,
    from: &str,
    to: &str,
    patched_count: &mut u32,
  ) -> XrfResult<Vec<u8>> {
    chunk.reset_pos()?;

    let mut buffer: Vec<u8> = Vec::new();

    for mut nested in chunk.read_children()? {
      let mut nested_buffer: Vec<u8> = Vec::new();

      for mut inner in nested.read_children()? {
        let payload: Vec<u8> = if inner.id == OgfTextureChunk::CHUNK_ID {
          let texture: OgfTextureChunk = OgfTextureChunk::read::<T, _>(&mut inner)?;

          let renamed: OgfTextureChunk = if texture.texture_name == from {
            *patched_count += 1;

            OgfTextureChunk {
              texture_name: String::from(to),
              shader_name: texture.shader_name,
            }
          } else {
            texture
          };

          let mut texture_writer: ChunkWriter = ChunkWriter::new();

          renamed.write::<T>(&mut texture_writer)?;
          texture_writer.flush_raw_into_buffer()?
        } else {
          inner.reset_pos()?;
          inner.read_remaining()?
        };

        let mut inner_writer: ChunkWriter = ChunkWriter::new();

        inner_writer.write_all(&payload)?;
        inner_writer.flush_chunk_into::<T>(&mut nested_buffer, inner.id)?;
      }

      let mut nested_writer: ChunkWriter = ChunkWriter::new();

      nested_writer.write_all(&nested_buffer)?;
      nested_writer.flush_chunk_into::<T>(&mut buffer, nested.id)?;
    }

    Ok(buffer)
  }

  /// Guard that a rename which changes nothing reproduces the source file, normalized.
  fn assert_chunk_copy_is_lossless<T: ByteOrder>(source: &Path, original: &[u8], from: &str) -> XrfResult {
    let (reverted, _) = Self::write_texture_refs_to_buffer::<T>(OgfRawPatch::open_source(source)?, from, from)?;

    OgfRawPatch::assert_chunk_copy_is_lossless::<T>(
      source,
      original,
      &reverted,
      "renaming a texture reference to itself",
    )
  }

  /// Verify the written file names the new reference and no longer names the old one.
  fn assert_written_refs_match<T: ByteOrder>(destination: &Path, from: &str, to: &str) -> XrfResult {
    let written: Vec<String> = OgfFile::read_texture_refs_from_path::<T, _>(&destination)?;

    if written.iter().any(|it| it == from) {
      return Err(XrfError::new_verify_error(format!(
        "Wrote {} but it still names '{}', refs are {:?}",
        format_path(destination),
        from,
        written
      )));
    }

    if !written.iter().any(|it| it == to) {
      return Err(XrfError::new_verify_error(format!(
        "Wrote {} but it does not name '{}', refs are {:?}",
        format_path(destination),
        to,
        written
      )));
    }

    Ok(())
  }

  /// Restore the destination after a failed verification, so a bad patch never survives.
  fn revert_destination(source: &Path, destination: &Path, original: &[u8]) -> XrfResult {
    if destination == source {
      fs::write(destination, original)?;
    } else if destination.exists() {
      fs::remove_file(destination)?;
    }

    Ok(())
  }
}
