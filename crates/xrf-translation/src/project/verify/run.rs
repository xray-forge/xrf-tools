use std::path::{Display, Path};
use std::time::Instant;

use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};

use crate::json::read::read_json;
use crate::language::TranslationLanguage;
use crate::project::verify::options::ProjectVerifyOptions;
use crate::project::verify::result::ProjectVerifyResult;
use crate::source_file_name::is_json_source;
use crate::types::TranslationJson;

/// Verify every translation source in a directory.
///
/// # Errors
///
/// Returns a read error when the tree cannot be walked, and a parsing error for an unreadable source.
pub fn verify_dir(dir: &Path, options: &ProjectVerifyOptions) -> XrfResult<ProjectVerifyResult> {
  log::info!("Verifying dir {}", dir.display());
  xrf_output::info!(options.output, "Verifying dir {}", dir.display());

  let started_at: Instant = Instant::now();
  let mut result: ProjectVerifyResult = ProjectVerifyResult::new();

  for entry in WalkDir::new(dir).sort_by_file_name() {
    let entry: DirEntry = entry.map_err(|error| {
      XrfError::new_read_error(format!(
        "Failed to walk translation directory '{}': {error}",
        dir.display()
      ))
    })?;

    let entry_path: &Path = entry.path();

    if entry_path.is_file() {
      result.merge(verify_file(&entry_path, options)?);
    }
  }

  result.duration = started_at.elapsed();

  log::info!(
    "Verified dir {} in {}",
    dir.display(),
    xrf_utils::format_duration(result.duration)
  );

  Ok(result)
}

/// Verify one source, skipping anything that is not a multi-language JSON.
///
/// # Errors
///
/// Returns a parsing error for an unreadable source.
pub fn verify_file<P: AsRef<Path>>(path: &P, options: &ProjectVerifyOptions) -> XrfResult<ProjectVerifyResult> {
  // Through the shared parser rather than comparing an extension, so this recognises the same names
  // the reader does. An exact compare skipped `ST_A.JSON` with only an info line, while the editor
  // opened it — the VFS lower-cases logical paths and the host walk does not.
  if is_json_source(path.as_ref()) {
    return verify_json_file(path, options);
  }

  log::info!("Skip file {}", path.as_ref().display());
  xrf_output::info!(options.output, "Skip file {}", path.as_ref().display());

  Ok(ProjectVerifyResult::new())
}

/// Record every id a requested language has no text for.
///
/// # Errors
///
/// Returns a parsing error for an unreadable source.
pub fn verify_json_file<P: AsRef<Path>>(path: &P, options: &ProjectVerifyOptions) -> XrfResult<ProjectVerifyResult> {
  let path_display: Display = path.as_ref().display();
  let mut result: ProjectVerifyResult = ProjectVerifyResult::new();

  log::info!("Verifying dynamic JSON file {}", path_display);

  let started_at: Instant = Instant::now();
  let parsed: TranslationJson = read_json(path.as_ref())?;

  let languages: Vec<String> = if options.language == TranslationLanguage::All {
    TranslationLanguage::get_all_strings()
  } else {
    vec![options.language.to_string()]
  };

  for language in languages {
    for (key, entry) in &parsed {
      // A present-but-null entry counts as missing: it is a placeholder waiting for a translator.
      let is_missing: bool = entry.get(&language).is_none_or(|translation| translation.is_none());

      if is_missing {
        xrf_output::error!(
          options.output,
          "Translation key missing: {} {} in {}",
          key,
          language,
          path_display
        );

        result.record_missing_translation(path.as_ref(), key, &language);
      }
    }
  }

  result.checked_translations_count = parsed.len() as u32;
  result.duration = started_at.elapsed();

  log::info!(
    "Verified file {} in {}",
    path_display,
    xrf_utils::format_duration(result.duration)
  );

  Ok(result)
}
