use std::collections::HashSet;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::source_file_name::is_json_source;

/// The sources a formatting run will read, from the paths it was given.
///
/// A directory is walked and filtered to JSON sources. A file named explicitly is taken whatever its extension, so a
/// caller can format one file the walk would not have picked; it still has to parse, which is where a mistake surfaces.
/// This is `ltx format`'s rule rather than `verify_file`'s and `initialize_file`'s, which skip a name they do not
/// recognise with an info line — silently skipping a file somebody typed by hand is the failure `parse`'s empty-scope
/// refusal exists to stop.
///
/// De-duplicated, because two paths may overlap, and sorted, because a walk order is not a name order and a run should
/// read the same on every machine.
///
/// # Errors
///
/// Returns a not-found error for a path that does not exist, a read error when a directory cannot be walked, and an
/// invalid error when the paths together select nothing. The last one is deliberate and differs from `ltx format`,
/// which formats an empty set successfully: a run that found nothing and a run that found everything already canonical
/// both exit 0 over an empty count, and the dangerous version of that is a renamed directory quietly making a check
/// gate vacuous.
pub(crate) fn select_sources(paths: &[PathBuf]) -> XrfResult<Vec<PathBuf>> {
  let mut files: Vec<PathBuf> = Vec::new();
  let mut visited: HashSet<PathBuf> = HashSet::new();

  for path in paths {
    select_path(path, &mut files, &mut visited)?;
  }

  if files.is_empty() {
    return Err(XrfError::new_invalid_error(format!(
      "No translation sources to format at {}. Check the path.",
      describe(paths)
    )));
  }

  files.sort();

  Ok(files)
}

/// Expand one path: a directory walked for JSON sources, a file taken as given.
fn select_path(path: &Path, files: &mut Vec<PathBuf>, visited: &mut HashSet<PathBuf>) -> XrfResult<()> {
  if path.is_dir() {
    for entry in WalkDir::new(path) {
      // A walk error without an io source — a filesystem loop — must fail cleanly rather than panic.
      let entry: DirEntry = entry.map_err(|error| {
        let message: String = error.to_string();

        match error.into_io_error() {
          Some(io_error) => XrfError::from(io_error),
          None => XrfError::new_read_error(message),
        }
      })?;

      let entry_path: &Path = entry.path();

      if entry_path.is_file() && is_json_source(entry_path) && visited.insert(entry_path.into()) {
        files.push(entry_path.into());
      }
    }

    return Ok(());
  }

  if !path.exists() {
    return Err(XrfError::new_not_found_error(format!(
      "Failed to format translations, provided path does not exist: {}",
      format_path(path)
    )));
  }

  if visited.insert(path.to_path_buf()) {
    files.push(path.to_path_buf());
  }

  Ok(())
}

/// Name the paths a refusal is about, so the message says which ones found nothing.
fn describe(paths: &[PathBuf]) -> String {
  paths
    .iter()
    .map(|path| format!("'{}'", format_path(path)))
    .collect::<Vec<String>>()
    .join(", ")
}
