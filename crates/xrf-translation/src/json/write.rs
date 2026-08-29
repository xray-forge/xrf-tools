use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::edit::TranslationEdit;
use crate::json::read::read_json;
use crate::staged_write::write_file_staged;
use crate::types::TranslationJson;

/// Apply edits to one language inside a multi-language JSON source.
///
/// # Errors
///
/// Returns a parsing error when the file cannot be read or re-serialized, and an IO error when it
/// cannot be replaced.
pub fn apply_edits(path: &Path, language: &str, edits: &[TranslationEdit]) -> XrfResult {
  let mut parsed: TranslationJson = read_json(path)?;

  for edit in edits {
    match edit {
      TranslationEdit::Set { id, value } => {
        // Stored as given: a multi-line value keeps its array form here, which is the only place that
        // form exists.
        parsed
          .entry(id.clone())
          .or_default()
          .insert(language.to_owned(), Some(value.clone()));
      }
      TranslationEdit::Remove { id } => {
        // One language, matching what removing an entry from an XML file does. Dropping every
        // language at once is a separate action the caller has to ask for by name.
        if let Some(entry) = parsed.get_mut(id) {
          entry.shift_remove(language);

          if entry.is_empty() {
            parsed.shift_remove(id);
          }
        }
      }
    }
  }

  let mut serialized: Vec<u8> = serde_json::to_vec_pretty(&parsed).map_err(|error| {
    XrfError::new_parsing_error(format!(
      "Failed to serialize translation JSON '{}': {error}",
      format_path(path)
    ))
  })?;

  if fs::read(path).is_ok_and(|original| original.ends_with(b"\n")) {
    serialized.push(b'\n');
  }

  write_file_staged(path, &serialized)
}
