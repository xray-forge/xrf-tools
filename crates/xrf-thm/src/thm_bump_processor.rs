use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::thm_bump_mode::ThmBumpMode;
use crate::thm_bump_patch_report::ThmBumpPatchReport;
use crate::thm_file::ThmFile;

/// Editing operations over the bump declaration of a thm file.
pub struct ThmBumpProcessor {}

impl ThmBumpProcessor {
  /// Repoint the bump declaration of a thm file, leaving its mode alone.
  pub fn patch_bump_name_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    bump_name: &str,
    is_dry_run: bool,
  ) -> XrfResult<ThmBumpPatchReport> {
    Self::patch_bump_to_path::<T>(source, destination, Some(bump_name), None, is_dry_run)
  }

  /// Turn the bump declaration of a thm file off, clearing the name the way the SDK does.
  pub fn patch_bump_off_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    is_dry_run: bool,
  ) -> XrfResult<ThmBumpPatchReport> {
    Self::patch_bump_to_path::<T>(source, destination, Some(""), Some(ThmBumpMode::None), is_dry_run)
  }

  /// Rewrite the bump declaration of a thm file, verifying the result before letting it survive.
  ///
  /// The source is read once, and every pass over it reads that one buffer. The patched bytes are checked before
  /// anything is written, so a refused patch leaves the destination as it was and a written one replaces it whole.
  pub fn patch_bump_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    bump_name: Option<&str>,
    mode: Option<ThmBumpMode>,
    is_dry_run: bool,
  ) -> XrfResult<ThmBumpPatchReport> {
    let original: Arc<Vec<u8>> = Arc::new(Self::read_source(source)?);
    let existing: ThmBumpChunk = Self::read_bump::<T>(source, &original)?;

    Self::assert_chunk_copy_is_lossless::<T>(source, &original, &existing)?;

    let patch: ThmBumpChunk = ThmBumpChunk {
      virtual_height: existing.virtual_height,
      mode: mode.unwrap_or(existing.mode),
      name: bump_name.map_or_else(|| existing.name.clone(), str::to_owned),
    };

    let patched: Arc<Vec<u8>> = Arc::new(Self::write_bump::<T>(
      ChunkReader::from_shared(Arc::clone(&original))?,
      &patch,
    )?);

    Self::assert_patched_bump_matches::<T>(source, &patched, &patch)?;

    let report: ThmBumpPatchReport = ThmBumpPatchReport {
      original_size: original.len(),
      patched_size: patched.len(),
      previous_name: existing.name,
      previous_mode: existing.mode,
      is_dry_run,
    };

    if !is_dry_run {
      StagedFile::write_creating_directories(destination, &patched)?;
    }

    Ok(report)
  }

  /// Rewrite the bump chunk of the thm file `bytes` hold, copying every other chunk verbatim.
  pub fn write_bump_name_to_buffer<T: ByteOrder>(bytes: Vec<u8>, bump: &ThmBumpChunk) -> XrfResult<Vec<u8>> {
    Self::write_bump::<T>(ChunkReader::from_vec(bytes)?, bump)
  }

  /// Rewrite the bump chunk of the thm file `reader` opens, copying every other chunk verbatim.
  fn write_bump<T: ByteOrder>(
    mut reader: ChunkReader<InMemoryChunkDataSource>,
    bump: &ThmBumpChunk,
  ) -> XrfResult<Vec<u8>> {
    let mut chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = reader.read_children_including_compressed()?;
    let mut buffer: Vec<u8> = Vec::new();
    let mut patched_count: u32 = 0;

    for chunk in &mut chunks {
      let payload: Vec<u8> = if chunk.id == ThmBumpChunk::CHUNK_ID {
        patched_count += 1;

        let mut bump_writer: ChunkWriter = ChunkWriter::new();

        bump.write::<T>(&mut bump_writer)?;

        bump_writer.flush_raw_into_buffer()?
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
        "Expected exactly one thm bump chunk to rewrite, got {patched_count}"
      )));
    }

    Ok(buffer)
  }

  /// The whole source, read once for every view a patch takes of it.
  fn read_source(source: &Path) -> XrfResult<Vec<u8>> {
    fs::read(source).map_err(|error| {
      XrfError::new_io_error(
        format!("THM file was not read: {}, error: {error}", format_path(source)),
        error.kind(),
      )
    })
  }

  /// The bump declaration of the thm file `bytes` hold, read from `source`.
  fn read_bump<T: ByteOrder>(source: &Path, bytes: &Arc<Vec<u8>>) -> XrfResult<ThmBumpChunk> {
    ThmFile::read_from_chunk::<T, _>(&mut ChunkReader::from_shared(Arc::clone(bytes))?)?
      .bump
      .ok_or_else(|| XrfError::new_not_found_error(format!("THM file declares no bump chunk: {}", format_path(source))))
  }

  /// Guard that rewriting the declaration a file already has reproduces that file byte for byte.
  fn assert_chunk_copy_is_lossless<T: ByteOrder>(
    source: &Path,
    original: &Arc<Vec<u8>>,
    existing: &ThmBumpChunk,
  ) -> XrfResult {
    let reverted: Vec<u8> = Self::write_bump::<T>(ChunkReader::from_shared(Arc::clone(original))?, existing)?;

    if reverted != **original {
      return Err(XrfError::new_verify_error(format!(
        "Refused to patch {}, rewriting its existing bump declaration did not reproduce the source file, {} bytes original and {} bytes rewritten",
        format_path(source),
        original.len(),
        reverted.len()
      )));
    }

    Ok(())
  }

  /// Guard that the patched bytes read back exactly the requested declaration.
  fn assert_patched_bump_matches<T: ByteOrder>(
    source: &Path,
    patched: &Arc<Vec<u8>>,
    expected: &ThmBumpChunk,
  ) -> XrfResult {
    let read_back: ThmBumpChunk =
      ThmFile::read_from_chunk::<T, _>(&mut ChunkReader::from_shared(Arc::clone(patched))?)?
        .bump
        .ok_or_else(|| XrfError::new_verify_error(format!("Patched {} declares no bump chunk", format_path(source))))?;

    if &read_back != expected {
      return Err(XrfError::new_verify_error(format!(
        "Patched {} reads back bump {:?} instead of {:?}",
        format_path(source),
        read_back,
        expected
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

  use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::GeneratedResources;

  use crate::chunks::thm_bump_chunk::ThmBumpChunk;
  use crate::thm_bump_mode::ThmBumpMode;
  use crate::thm_bump_processor::ThmBumpProcessor;
  use crate::thm_file::ThmFile;

  /// Payload standing in for a chunk the writer must copy verbatim.
  const OPAQUE_PAYLOAD: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];

  /// An id the format has no name for, so the copy has to carry it through untouched.
  const OPAQUE_CHUNK_ID: u32 = 0x0820;

  fn write_sample(filename: &str, bump: &ThmBumpChunk) -> XrfResult<PathBuf> {
    let mut opaque_writer: ChunkWriter = ChunkWriter::new();
    opaque_writer.write_all(&OPAQUE_PAYLOAD)?;

    let mut bump_writer: ChunkWriter = ChunkWriter::new();
    bump.write::<XRayByteOrder>(&mut bump_writer)?;

    let mut file: File = GeneratedResources::overwrite(filename)?;

    opaque_writer.flush_chunk_into::<XRayByteOrder>(&mut file, OPAQUE_CHUNK_ID)?;
    bump_writer.flush_chunk_into::<XRayByteOrder>(&mut file, ThmBumpChunk::CHUNK_ID)?;

    Ok(GeneratedResources::path(filename))
  }

  fn used_bump(name: &str) -> ThmBumpChunk {
    ThmBumpChunk {
      virtual_height: 0.05,
      mode: ThmBumpMode::Use,
      name: name.to_owned(),
    }
  }

  #[test]
  fn test_write_bump_reproduces_source_when_unchanged() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "unchanged.thm");
    let bump: ThmBumpChunk = used_bump("wpn\\pistols\\wpn_pm\\wpn_pm_bump");
    let path: PathBuf = write_sample(&filename, &bump)?;

    assert_eq!(
      ThmBumpProcessor::write_bump_name_to_buffer::<XRayByteOrder>(fs::read(&path)?, &bump)?,
      fs::read(&path)?,
      "Expect rewriting the existing declaration to reproduce source bytes"
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_repoints_name_and_preserves_other_chunks() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "repointed.thm");
    let path: PathBuf = write_sample(&filename, &used_bump("wpn\\pistols\\wpn_pm\\wpn_pm_bump"))?;
    let original: Vec<u8> = fs::read(&path)?;

    ThmBumpProcessor::patch_bump_name_to_path::<XRayByteOrder>(&path, &path, "wpn\\wpn_pm\\wpn_pm_bump", false)?;

    let patched: Vec<u8> = fs::read(&path)?;
    let opaque_len: usize = 8 + OPAQUE_PAYLOAD.len();

    assert_eq!(
      patched[..opaque_len],
      original[..opaque_len],
      "Expect leading authoring chunk to be copied verbatim"
    );

    let thm: ThmFile = ThmFile::read_from_path::<XRayByteOrder, _>(&path)?;

    assert_eq!(thm.used_bump_name(), Some("wpn\\wpn_pm\\wpn_pm_bump"));
    assert_eq!(
      thm.bump.as_ref().map(|bump| bump.mode),
      Some(ThmBumpMode::Use),
      "Expect mode to survive repointing"
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_is_a_no_op_on_dry_run() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "dry_run.thm");
    let path: PathBuf = write_sample(&filename, &used_bump("wpn\\source\\name_bump"))?;
    let original: Vec<u8> = fs::read(&path)?;

    let report = ThmBumpProcessor::patch_bump_name_to_path::<XRayByteOrder>(&path, &path, "wpn\\other_bump", true)?;

    assert!(report.is_dry_run);
    assert_eq!(fs::read(&path)?, original, "Expect dry run to leave the file untouched");

    Ok(())
  }

  #[test]
  fn test_patch_bump_off_clears_mode_and_name() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "turned_off.thm");
    let path: PathBuf = write_sample(&filename, &used_bump("tile\\tile_walls_red_01_bump"))?;

    let report = ThmBumpProcessor::patch_bump_off_to_path::<XRayByteOrder>(&path, &path, false)?;

    assert_eq!(report.previous_name, "tile\\tile_walls_red_01_bump");
    assert_eq!(report.previous_mode, ThmBumpMode::Use);

    let thm: ThmFile = ThmFile::read_from_path::<XRayByteOrder, _>(&path)?;

    assert_eq!(
      thm.used_bump_name(),
      None,
      "Expect a disabled declaration to resolve to no bump"
    );
    assert_eq!(
      thm.bump.as_ref().map(|bump| (bump.mode, bump.name.as_str())),
      Some((ThmBumpMode::None, "")),
      "Expect the SDK form of a disabled bump, mode none and an empty name"
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_keeps_omitted_fields() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "partial.thm");
    let path: PathBuf = write_sample(&filename, &used_bump("wpn\\source_bump"))?;

    // Mode omitted, so the parallax variant must survive a rename.
    ThmBumpProcessor::patch_bump_to_path::<XRayByteOrder>(&path, &path, None, Some(ThmBumpMode::UseParallax), false)?;
    ThmBumpProcessor::patch_bump_to_path::<XRayByteOrder>(&path, &path, Some("wpn\\renamed_bump"), None, false)?;

    let thm: ThmFile = ThmFile::read_from_path::<XRayByteOrder, _>(&path)?;

    assert_eq!(
      thm.bump.as_ref().map(|bump| (bump.mode, bump.name.as_str())),
      Some((ThmBumpMode::UseParallax, "wpn\\renamed_bump"))
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_requires_bump_chunk() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "without_bump.thm");
    let mut opaque_writer: ChunkWriter = ChunkWriter::new();

    opaque_writer.write_all(&OPAQUE_PAYLOAD)?;
    opaque_writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0x0812)?;

    let path: PathBuf = GeneratedResources::path(&filename);

    assert!(
      ThmBumpProcessor::patch_bump_name_to_path::<XRayByteOrder>(&path, &path, "any", false).is_err(),
      "Expect patch to be refused when the file declares no bump"
    );

    Ok(())
  }

  #[test]
  fn test_unused_modes_report_no_bump_name() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "unused_mode.thm");
    let path: PathBuf = write_sample(
      &filename,
      &ThmBumpChunk {
        virtual_height: 0.05,
        mode: ThmBumpMode::None,
        name: String::from("wpn\\ignored_bump"),
      },
    )?;

    assert_eq!(
      ThmFile::read_from_path::<XRayByteOrder, _>(&path)?.used_bump_name(),
      None,
      "Expect a declaration the engine ignores to resolve to no bump"
    );

    Ok(())
  }
}
