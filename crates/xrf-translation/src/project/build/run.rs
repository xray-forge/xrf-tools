use std::ffi::OsStr;
use std::fs::File;
use std::io::{Write, copy};
use std::path::{Display, Path, PathBuf};
use std::time::Instant;

use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::encode_string_to_bytes;

use crate::json;
use crate::json::read::read_json;
use crate::language::TranslationLanguage;
use crate::project::build::compile::compile_by_language;
use crate::project::build::options::ProjectBuildOptions;
use crate::project::build::result::ProjectBuildResult;
use crate::project::build::targets::{ensure_output_outside_source, prepare_target_file, validate_targets};
use crate::types::TranslationJson;
use crate::xml;

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

/// Build one source, dispatching on what kind of file it is.
///
/// # Errors
///
/// Returns whatever building the XML or JSON source returns.
pub fn build_file<P: AsRef<Path>>(path: &P, options: &ProjectBuildOptions) -> XrfResult<ProjectBuildResult> {
  let extension: Option<&OsStr> = path.as_ref().extension();
  let started_at: Instant = Instant::now();

  let mut result: ProjectBuildResult = ProjectBuildResult::new();

  if let Some(extension) = extension {
    if extension == xml::FILE_EXTENSION {
      build_xml_file(path, options)?;
    } else if extension == json::FILE_EXTENSION {
      build_json_file(path, options)?;
    } else {
      log::info!("Skip file {}", path.as_ref().display());
      xrf_output::info!(options.output, "Skip file {}", path.as_ref().display());
    }
  }

  result.duration = started_at.elapsed();

  log::info!(
    "Built file {} in {}",
    path.as_ref().display(),
    xrf_utils::format_duration(result.duration)
  );

  Ok(result)
}

/// Copy an XML source into the languages it belongs to.
///
/// A language-suffixed file goes to that language alone. An unsuffixed one is language-neutral and is
/// copied into every language, which is why it must never be presented as English anywhere else.
///
/// # Errors
///
/// Returns an IO error when a source cannot be read or a target cannot be written.
pub fn build_xml_file<P: AsRef<Path>>(path: &P, options: &ProjectBuildOptions) -> XrfResult {
  let path_display: Display = path.as_ref().display();
  let locale: Option<TranslationLanguage> = path
    .as_ref()
    .file_name()
    .and_then(|it| it.to_str())
    .and_then(TranslationLanguage::from_file_name);

  if let Some(locale) = locale {
    xrf_output::info!(options.output, "Building XML based translations {path_display}");

    if options.language == TranslationLanguage::All || locale == options.language {
      log::info!("Building dynamic XML file {} ({})", path_display, locale);

      copy(
        &mut File::open(path)?,
        &mut prepare_target_file(path, &options.output_dir, &locale, options)?,
      )?;
    } else {
      log::info!("Skip dynamic XML file {}", path_display);
    }
  } else {
    log::info!("Building static XML file {}", path.as_ref().display());

    xrf_output::info!(options.output, "Copy static XML translations {path_display}");

    if options.language == TranslationLanguage::All {
      for language in TranslationLanguage::get_all() {
        copy(
          &mut File::open(path)?,
          &mut prepare_target_file(path, &options.output_dir, &language, options)?,
        )?;
      }
    } else {
      copy(
        &mut File::open(path)?,
        &mut prepare_target_file(path, &options.output_dir, &options.language, options)?,
      )?;
    }
  }

  Ok(())
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
