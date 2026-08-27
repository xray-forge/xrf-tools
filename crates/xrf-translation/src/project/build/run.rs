use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::encode_string_to_bytes;

use crate::json::read::read_json;
use crate::language::TranslationLanguage;
use crate::project::build::compile::compile_by_language;
use crate::project::build::options::ProjectBuildOptions;
use crate::project::build::result::ProjectBuildResult;
use crate::project::build::targets::{ensure_output_outside_source, prepare_target_file, validate_targets};
use crate::source_file_name::is_json_source;
use crate::types::TranslationJson;

/// Build every translation source in a directory.
///
/// Targets are validated before anything is written, so a build that would have two sources fighting
/// over one output file fails having produced nothing.
///
/// # Errors
///
/// Returns an invalid error for colliding targets or output inside the sources, and whatever building
/// an individual file returns.
pub fn build_dir(dir: &Path, options: &ProjectBuildOptions) -> XrfResult<ProjectBuildResult> {
  log::info!("Building dir {}", dir.display());
  xrf_output::info!(options.output, "Building dir {}", dir.display());

  let started_at: Instant = Instant::now();
  let mut result: ProjectBuildResult = ProjectBuildResult::new();
  let mut source_files: Vec<PathBuf> = Vec::new();

  ensure_output_outside_source(dir, &options.output_dir)?;

  for entry in WalkDir::new(dir).sort_by_file_name() {
    let entry: DirEntry = entry.map_err(|error| {
      XrfError::new_read_error(format!(
        "Failed to walk translation directory '{}': {error}",
        dir.display()
      ))
    })?;

    if entry.path().is_file() {
      source_files.push(entry.into_path());
    }
  }

  validate_targets(&source_files, options)?;

  for source_file in source_files {
    build_file(&source_file, options)?;
  }

  result.duration = started_at.elapsed();

  log::info!(
    "Built dir {} in {}",
    dir.display(),
    xrf_utils::format_duration(result.duration)
  );

  Ok(result)
}

/// Build one source, skipping anything that is not a multi-language JSON.
///
/// # Errors
///
/// Returns whatever building the JSON source returns.
pub fn build_file<P: AsRef<Path>>(path: &P, options: &ProjectBuildOptions) -> XrfResult<ProjectBuildResult> {
  let started_at: Instant = Instant::now();

  let mut result: ProjectBuildResult = ProjectBuildResult::new();

  // Through the shared parser rather than comparing an extension; see `verify_file` for why.
  if is_json_source(path.as_ref()) {
    build_json_file(path, options)?;
  } else {
    log::info!("Skip file {}", path.as_ref().display());
    xrf_output::info!(options.output, "Skip file {}", path.as_ref().display());
  }

  result.duration = started_at.elapsed();

  log::info!(
    "Built file {} in {}",
    path.as_ref().display(),
    xrf_utils::format_duration(result.duration)
  );

  Ok(result)
}

/// Compile a multi-language JSON source into one string table per language.
///
/// # Errors
///
/// Returns a parsing error for an unreadable source and an encoding error for a value a target
/// language cannot hold.
pub fn build_json_file<P: AsRef<Path>>(path: &P, options: &ProjectBuildOptions) -> XrfResult {
  xrf_output::info!(
    options.output,
    "Building JSON based translations {}",
    path.as_ref().display()
  );

  let parsed: TranslationJson = read_json(path.as_ref())?;

  if options.language == TranslationLanguage::All {
    for language in TranslationLanguage::get_all() {
      build_json_by_language(path.as_ref(), &parsed, &language, options)?;
    }
  } else {
    build_json_by_language(path.as_ref(), &parsed, &options.language, options)?;
  }

  Ok(())
}

fn build_json_by_language(
  path: &Path,
  source: &TranslationJson,
  language: &TranslationLanguage,
  options: &ProjectBuildOptions,
) -> XrfResult {
  let data: Vec<u8> = encode_string_to_bytes(
    &compile_by_language(path, source, language, options)?,
    language.new_language_encoder(),
  )?;

  prepare_target_file(&path, &options.output_dir, language, options)?.write_all(&data)?;

  Ok(())
}
