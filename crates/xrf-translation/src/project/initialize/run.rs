use std::path::{Display, Path};
use std::time::Instant;

use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};

use crate::json::read::read_json;
use crate::language::TranslationLanguage;
use crate::project::initialize::options::ProjectInitializeOptions;
use crate::project::initialize::result::ProjectInitializeResult;
use crate::source_file_name::is_json_source;
use crate::staged_write::write_file_staged;
use crate::types::TranslationJson;

/// Initialize every translation source in a directory.
///
/// # Errors
///
/// Returns a read error when the tree cannot be walked, and whatever initializing a file returns.
pub fn initialize_dir<P: AsRef<Path>>(
  dir: &P,
  options: &ProjectInitializeOptions,
) -> XrfResult<ProjectInitializeResult> {
  xrf_output::info!(options.output, "Initializing dir {}", dir.as_ref().display());

  let started_at: Instant = Instant::now();
  let mut result: ProjectInitializeResult = ProjectInitializeResult::new();

  for entry in WalkDir::new(dir).sort_by_file_name() {
    let entry: DirEntry = entry.map_err(|error| {
      XrfError::new_read_error(format!(
        "Failed to walk translation directory '{}': {error}",
        dir.as_ref().display()
      ))
    })?;

    if entry.path().is_file() {
      initialize_file(&entry.path(), options)?;
    }
  }

  result.duration = started_at.elapsed();

  log::info!(
    "Initialize dir {} in {}",
    dir.as_ref().display(),
    xrf_utils::format_duration(result.duration)
  );

  Ok(result)
}

/// Initialize one source, skipping anything that is not a multi-language JSON.
///
/// # Errors
///
/// Returns whatever initializing the JSON source returns.
pub fn initialize_file<P: AsRef<Path>>(
  path: &P,
  options: &ProjectInitializeOptions,
) -> XrfResult<ProjectInitializeResult> {
  // Through the shared parser rather than comparing an extension; see `verify_file` for why.
  if is_json_source(path.as_ref()) {
    return initialize_json_file(path, options);
  }

  log::info!("Skip file {}", path.as_ref().display());
  xrf_output::info!(options.output, "Skip file {}", path.as_ref().display());

  Ok(ProjectInitializeResult::new())
}

/// Give every id an explicit null for each language it has no text for.
///
/// The file is only replaced when something was actually added, so running this over an already
/// complete project rewrites nothing.
///
/// # Errors
///
/// Returns a parsing error for an unreadable source, a serialization error when the result cannot be
/// produced, and an IO error when the file cannot be replaced.
pub fn initialize_json_file<P: AsRef<Path>>(
  path: &P,
  options: &ProjectInitializeOptions,
) -> XrfResult<ProjectInitializeResult> {
  let path_display: Display = path.as_ref().display();

  let mut result: ProjectInitializeResult = ProjectInitializeResult::new();
  let mut initialized_count: u32 = 0;

  log::info!("Initializing dynamic JSON file {}", path_display);

  let started_at: Instant = Instant::now();
  let mut parsed: TranslationJson = read_json(path.as_ref())?;

  let all_languages: Vec<String> = TranslationLanguage::get_all_strings();

  for (key, value) in &mut parsed {
    for language in &all_languages {
      if !value.contains_key(language) {
        initialized_count += 1;

        log::info!("Initializing missing key: {key} - {language}");
        xrf_output::info!(options.output, "Initializing missing key: {key} - {language}");

        value.insert(String::from(language), None);
      }
    }
  }

  if initialized_count > 0 {
    let serialized: Vec<u8> = serde_json::to_vec_pretty(&parsed).map_err(|error| {
      XrfError::new_serialization_error(format!(
        "Failed to serialize initialized translation JSON '{}': {error}",
        path_display
      ))
    })?;

    write_file_staged(path.as_ref(), &serialized)?;
  }

  result.duration = started_at.elapsed();

  if initialized_count > 0 {
    log::info!(
      "Initialized file {} in {}, {} keys added",
      path_display,
      xrf_utils::format_duration(result.duration),
      initialized_count
    );
  } else {
    log::info!(
      "Skip file {}, checked in {}",
      path_display,
      xrf_utils::format_duration(result.duration)
    );
  }

  Ok(result)
}
