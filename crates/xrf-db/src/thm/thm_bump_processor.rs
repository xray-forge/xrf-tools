use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, open_export_file};

use crate::thm::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::thm::thm_bump_patch_report::ThmBumpPatchReport;
use crate::thm::thm_file::ThmFile;

/// Editing operations over the bump declaration of a thm file.
pub struct ThmBumpProcessor {}

impl ThmBumpProcessor {
  /// Repoint the bump declaration of a thm file, leaving its mode alone.
  ///
  /// Needed whenever a texture is imported under a different path than it had at its source: the
  /// thm names its bump absolutely, so the copied descriptor keeps pointing into the source layout
  /// and the engine resolves nothing.
  pub fn patch_bump_name_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    bump_name: &str,
    is_dry_run: bool,
  ) -> XrfResult<ThmBumpPatchReport> {
    Self::patch_bump_to_path::<T>(source, destination, Some(bump_name), None, is_dry_run)
  }

  /// Turn the bump declaration of a thm file off, clearing the name the way the SDK does.
  ///
  /// The honest fix for a descriptor asking for a bump that does not exist and is not going to.
  /// Leaving the name in place with the mode off would still be inert, but an empty name is what
  /// `STextureParams` writes for [`ThmBumpChunk::MODE_NONE`], so it stays diffable against vanilla.
  pub fn patch_bump_off_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    is_dry_run: bool,
  ) -> XrfResult<ThmBumpPatchReport> {
    Self::patch_bump_to_path::<T>(source, destination, Some(""), Some(ThmBumpChunk::MODE_NONE), is_dry_run)
  }

  /// Rewrite the bump declaration of a thm file, verifying the result before letting it survive.
  ///
  /// `bump_name` and `mode` each keep the file's existing value when omitted.
  ///
  /// Guards, in order: rewriting the file's existing declaration must reproduce the source bytes
  /// exactly, the patched file must read back what was requested, and a failure of either leaves
  /// neither a corrupted source nor a partial destination behind.
  pub fn patch_bump_to_path<T: ByteOrder>(
    source: &Path,
    destination: &Path,
    bump_name: Option<&str>,
    mode: Option<u32>,
    is_dry_run: bool,
  ) -> XrfResult<ThmBumpPatchReport> {
    let original: Vec<u8> = fs::read(source)?;
    let existing: ThmBumpChunk = Self::read_bump(source)?;

    Self::assert_chunk_copy_is_lossless::<T>(source, &original, &existing)?;

    let patch: ThmBumpChunk = ThmBumpChunk {
      virtual_height: existing.virtual_height,
      mode: mode.unwrap_or(existing.mode),
      name: bump_name.map_or_else(|| existing.name.clone(), str::to_owned),
    };

    let patched: Vec<u8> = Self::write_bump_name_to_buffer::<T>(Self::open_source(source)?, &patch)?;

    let report: ThmBumpPatchReport = ThmBumpPatchReport {
      original_size: original.len(),
      patched_size: patched.len(),
      previous_name: existing.name,
      previous_mode: existing.mode,
      is_dry_run,
    };

    if is_dry_run {
      return Ok(report);
    }

    if let Some(parent) = destination.parent() {
      fs::create_dir_all(parent)?;
    }

    open_export_file(destination)?.write_all(&patched)?;

    if let Err(error) = Self::assert_written_bump_matches(destination, &patch) {
      Self::revert_destination(source, destination, &original)?;

      return Err(error);
    }

    Ok(report)
  }

  /// Rewrite the bump chunk of a thm file, copying every other chunk verbatim.
  pub fn write_bump_name_to_buffer<T: ByteOrder>(file: File, bump: &ThmBumpChunk) -> XrfResult<Vec<u8>> {
    let mut chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_file(file)?.read_children()?;
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

  fn open_source(source: &Path) -> XrfResult<File> {
    File::open(source).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "THM file was not read: {}, error: {}",
        format_path(source),
        error
      ))
    })
  }

  fn read_bump(source: &Path) -> XrfResult<ThmBumpChunk> {
    ThmFile::read_from_path::<xrf_chunk::XRayByteOrder, _>(&source)?
      .bump
      .ok_or_else(|| XrfError::new_not_found_error(format!("THM file declares no bump chunk: {}", format_path(source))))
  }

  /// Guard that rewriting the declaration a file already has reproduces that file byte for byte.
  ///
  /// A thm carries authoring chunks this crate does not parse, so this is what proves the chunk
  /// copy preserves everything outside the bump chunk.
  fn assert_chunk_copy_is_lossless<T: ByteOrder>(source: &Path, original: &[u8], existing: &ThmBumpChunk) -> XrfResult {
    let reverted: Vec<u8> = Self::write_bump_name_to_buffer::<T>(Self::open_source(source)?, existing)?;

    if reverted != original {
      return Err(XrfError::new_verify_error(format!(
        "Refused to patch {}, rewriting its existing bump declaration did not reproduce the source file, {} bytes original and {} bytes rewritten",
        format_path(source),
        original.len(),
        reverted.len()
      )));
    }

    Ok(())
  }

  /// Guard that the written file reads back exactly the requested declaration.
  fn assert_written_bump_matches(destination: &Path, expected: &ThmBumpChunk) -> XrfResult {
    let read_back: ThmBumpChunk = Self::read_bump(destination)?;

    if &read_back != expected {
      return Err(XrfError::new_verify_error(format!(
        "Patched {} reads back bump {:?} instead of {:?}",
        format_path(destination),
        read_back,
        expected
      )));
    }

    Ok(())
  }

  /// Undo a failed write, leaving neither a corrupted source nor a partial destination behind.
  fn revert_destination(source: &Path, destination: &Path, original: &[u8]) -> XrfResult {
    if destination == source {
      fs::write(destination, original)?;
    } else if destination.exists() {
      fs::remove_file(destination)?;
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
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    overwrite_generated_test_resource_as_file,
  };

  use crate::thm::chunks::thm_bump_chunk::ThmBumpChunk;
  use crate::thm::thm_bump_processor::ThmBumpProcessor;
  use crate::thm::thm_file::ThmFile;

  /// Payload standing in for an authoring chunk the writer must copy verbatim.
  const OPAQUE_PAYLOAD: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];

  /// An authoring chunk this crate does not read, so the copy has to carry it through untouched.
  const OPAQUE_CHUNK_ID: u32 = 0x0819;

  fn write_sample(filename: &str, bump: &ThmBumpChunk) -> XrfResult<PathBuf> {
    let mut opaque_writer: ChunkWriter = ChunkWriter::new();
    opaque_writer.write_all(&OPAQUE_PAYLOAD)?;

    let mut bump_writer: ChunkWriter = ChunkWriter::new();
    bump.write::<XRayByteOrder>(&mut bump_writer)?;

    let mut file: File = overwrite_generated_test_resource_as_file(filename)?;

    opaque_writer.flush_chunk_into::<XRayByteOrder>(&mut file, OPAQUE_CHUNK_ID)?;
    bump_writer.flush_chunk_into::<XRayByteOrder>(&mut file, ThmBumpChunk::CHUNK_ID)?;

    Ok(build_absolute_generated_test_resource_path(filename))
  }

  fn used_bump(name: &str) -> ThmBumpChunk {
    ThmBumpChunk {
      virtual_height: 0.05,
      mode: ThmBumpChunk::MODE_USE,
      name: name.to_owned(),
    }
  }

  #[test]
  fn test_write_bump_reproduces_source_when_unchanged() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "unchanged.thm");
    let bump: ThmBumpChunk = used_bump("wpn\\pistols\\wpn_pm\\wpn_pm_bump");
    let path: PathBuf = write_sample(&filename, &bump)?;

    assert_eq!(
      ThmBumpProcessor::write_bump_name_to_buffer::<XRayByteOrder>(File::open(&path)?, &bump)?,
      fs::read(&path)?,
      "Expect rewriting the existing declaration to reproduce source bytes"
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_repoints_name_and_preserves_other_chunks() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "repointed.thm");
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
      Some(ThmBumpChunk::MODE_USE),
      "Expect mode to survive repointing"
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_is_a_no_op_on_dry_run() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "dry_run.thm");
    let path: PathBuf = write_sample(&filename, &used_bump("wpn\\source\\name_bump"))?;
    let original: Vec<u8> = fs::read(&path)?;

    let report = ThmBumpProcessor::patch_bump_name_to_path::<XRayByteOrder>(&path, &path, "wpn\\other_bump", true)?;

    assert!(report.is_dry_run);
    assert_eq!(fs::read(&path)?, original, "Expect dry run to leave the file untouched");

    Ok(())
  }

  #[test]
  fn test_patch_bump_off_clears_mode_and_name() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "turned_off.thm");
    let path: PathBuf = write_sample(&filename, &used_bump("tile\\tile_walls_red_01_bump"))?;

    let report = ThmBumpProcessor::patch_bump_off_to_path::<XRayByteOrder>(&path, &path, false)?;

    assert_eq!(report.previous_name, "tile\\tile_walls_red_01_bump");
    assert_eq!(report.previous_mode, ThmBumpChunk::MODE_USE);

    let thm: ThmFile = ThmFile::read_from_path::<XRayByteOrder, _>(&path)?;

    assert_eq!(
      thm.used_bump_name(),
      None,
      "Expect a disabled declaration to resolve to no bump"
    );
    assert_eq!(
      thm.bump.as_ref().map(|bump| (bump.mode, bump.name.as_str())),
      Some((ThmBumpChunk::MODE_NONE, "")),
      "Expect the SDK form of a disabled bump, mode none and an empty name"
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_keeps_omitted_fields() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "partial.thm");
    let path: PathBuf = write_sample(&filename, &used_bump("wpn\\source_bump"))?;

    // Mode omitted, so the parallax variant must survive a rename.
    ThmBumpProcessor::patch_bump_to_path::<XRayByteOrder>(
      &path,
      &path,
      None,
      Some(ThmBumpChunk::MODE_USE_PARALLAX),
      false,
    )?;
    ThmBumpProcessor::patch_bump_to_path::<XRayByteOrder>(&path, &path, Some("wpn\\renamed_bump"), None, false)?;

    let thm: ThmFile = ThmFile::read_from_path::<XRayByteOrder, _>(&path)?;

    assert_eq!(
      thm.bump.as_ref().map(|bump| (bump.mode, bump.name.as_str())),
      Some((ThmBumpChunk::MODE_USE_PARALLAX, "wpn\\renamed_bump"))
    );

    Ok(())
  }

  #[test]
  fn test_patch_bump_requires_bump_chunk() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "without_bump.thm");
    let mut opaque_writer: ChunkWriter = ChunkWriter::new();

    opaque_writer.write_all(&OPAQUE_PAYLOAD)?;
    opaque_writer
      .flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0x0812)?;

    let path: PathBuf = build_absolute_generated_test_resource_path(&filename);

    assert!(
      ThmBumpProcessor::patch_bump_name_to_path::<XRayByteOrder>(&path, &path, "any", false).is_err(),
      "Expect patch to be refused when the file declares no bump"
    );

    Ok(())
  }

  #[test]
  fn test_unused_modes_report_no_bump_name() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "unused_mode.thm");
    let path: PathBuf = write_sample(
      &filename,
      &ThmBumpChunk {
        virtual_height: 0.05,
        mode: ThmBumpChunk::MODE_NONE,
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
