use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::chunks::ogf_children_chunk::OgfChildrenChunk;
use crate::chunks::ogf_texture_chunk::OgfTextureChunk;
use crate::ogf_file::OgfFile;
use crate::ogf_raw_patch::OgfRawPatch;
use crate::ogf_refs_patch_report::OgfRefsPatchReport;
use crate::residue::{OgfNormalization, OgfResidue};

/// Editing operations over the texture refs of an ogf file.
pub struct OgfTextureRefsProcessor {}

impl OgfTextureRefsProcessor {
  /// Rename a texture reference of an ogf file, verifying the result before letting it survive.
  pub fn patch_texture_refs_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    from: &str,
    to: &str,
    is_dry_run: bool,
  ) -> XrfResult<OgfRefsPatchReport> {
    let original: Arc<Vec<u8>> = Arc::new(OgfRawPatch::read_source(source)?);
    let existing: Vec<String> =
      OgfFile::read_texture_refs_from_chunk::<T, _>(&mut ChunkReader::from_shared(Arc::clone(&original))?)?;

    Self::assert_chunk_copy_is_lossless::<T>(source, &original, from)?;

    let (patched, patched_count) =
      Self::write_texture_refs::<T>(ChunkReader::from_shared(Arc::clone(&original))?, from, to)?;
    let patched: Arc<Vec<u8>> = Arc::new(patched);

    if patched_count == 0 {
      return Err(XrfError::new_verify_error(format!(
        "Refused to patch {}, no texture reference matched '{}', found {:?}",
        format_path(source),
        from,
        existing
      )));
    }

    Self::assert_patched_refs_match::<T>(source, &patched, from, to)?;

    let report: OgfRefsPatchReport = OgfRefsPatchReport {
      original_size: original.len(),
      patched_size: patched.len(),
      patched_count,
      discarded_size: original
        .len()
        .saturating_sub(OgfNormalization::normalize_bytes::<T>(&original)?.len()),
      is_dry_run,
      previous_refs: existing,
    };

    if !is_dry_run {
      StagedFile::write_creating_directories(destination, &patched)?;
    }

    Ok(report)
  }

  /// Rename texture references of the ogf file `bytes` hold, copying every other chunk verbatim.
  pub fn write_texture_refs_to_buffer<T: ByteOrder>(bytes: Vec<u8>, from: &str, to: &str) -> XrfResult<(Vec<u8>, u32)> {
    Self::write_texture_refs::<T>(ChunkReader::from_vec(bytes)?, from, to)
  }

  /// Rename texture references of the ogf file `reader` opens, copying every other chunk verbatim.
  fn write_texture_refs<T: ByteOrder>(
    mut reader: ChunkReader<InMemoryChunkDataSource>,
    from: &str,
    to: &str,
  ) -> XrfResult<(Vec<u8>, u32)> {
    let (mut chunks, _) = OgfResidue::read_root_chunks::<T, _>(&mut reader)?;
    let mut buffer: Vec<u8> = Vec::new();
    let mut patched_count: u32 = 0;

    for chunk in &mut chunks {
      let payload: Vec<u8> = if chunk.id == OgfChildrenChunk::CHUNK_ID {
        Self::rename_children_texture_refs::<T, _>(chunk, from, to, &mut patched_count)?
      } else if OgfRawPatch::is_kinematics_chunk(chunk.id) {
        OgfRawPatch::read_normalized_kinematics_payload::<T, _>(chunk)?
      } else {
        OgfRawPatch::read_payload(chunk)?
      };

      OgfRawPatch::write_chunk::<T>(&mut buffer, chunk.id, payload)?;
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
              ..texture
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
  fn assert_chunk_copy_is_lossless<T: ByteOrder>(source: &Path, original: &Arc<Vec<u8>>, from: &str) -> XrfResult {
    let (reverted, _) = Self::write_texture_refs::<T>(ChunkReader::from_shared(Arc::clone(original))?, from, from)?;

    OgfRawPatch::assert_chunk_copy_is_lossless::<T>(
      source,
      original,
      &reverted,
      "renaming a texture reference to itself",
    )
  }

  /// Verify the patched bytes name the new reference and no longer name the old one.
  fn assert_patched_refs_match<T: ByteOrder>(source: &Path, patched: &Arc<Vec<u8>>, from: &str, to: &str) -> XrfResult {
    let patched_refs: Vec<String> =
      OgfFile::read_texture_refs_from_chunk::<T, _>(&mut ChunkReader::from_shared(Arc::clone(patched))?)?;

    if patched_refs.iter().any(|it| it == from) {
      return Err(XrfError::new_verify_error(format!(
        "Patched {} still names '{}', refs are {:?}",
        format_path(source),
        from,
        patched_refs
      )));
    }

    if !patched_refs.iter().any(|it| it == to) {
      return Err(XrfError::new_verify_error(format!(
        "Patched {} does not name '{}', refs are {:?}",
        format_path(source),
        to,
        patched_refs
      )));
    }

    Ok(())
  }
}
