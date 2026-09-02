use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{LineEndings, format_path};

use crate::edit::TranslationEdit;
use crate::json::normalize::{resolve_line_endings, to_canonical_bytes};
use crate::json::read::read_json;
use crate::types::TranslationJson;
use xrf_utils::{write_file_staged, write_new_file_staged};

/// What a canonical rewrite of one source would produce, beside what is there now.
pub(crate) struct CanonicalRender {
  /// The bytes a canonical source holds.
  pub bytes: Vec<u8>,
  /// What the file holds now, or nothing when it is not there yet.
  pub existing: Option<Vec<u8>>,
}

impl CanonicalRender {
  /// Whether writing this would change the file.
  pub(crate) fn is_changed(&self) -> bool {
    self.existing.as_deref() != Some(self.bytes.as_slice())
  }

  /// Whether the file is not there yet.
  pub(crate) fn is_new(&self) -> bool {
    self.existing.is_none()
  }
}

/// Render what one source would hold, without writing anything.
///
/// `line_endings` is the caller's explicit choice; `None` preserves the file's own convention.
///
/// # Errors
///
/// Returns an IO error when an existing file cannot be read — which is not the same as it being absent — and a
/// serialization error when the document cannot be rendered.
pub(crate) fn render_canonical(
  path: &Path,
  document: &TranslationJson,
  line_endings: Option<LineEndings>,
) -> XrfResult<CanonicalRender> {
  let existing: Option<Vec<u8>> = match fs::read(path) {
    Ok(bytes) => Some(bytes),
    // Absent is a state, not a failure: the importer creates sources that were never there.
    Err(error) if error.kind() == ErrorKind::NotFound => None,
    Err(error) => return Err(XrfError::from(error)),
  };

  let bytes: Vec<u8> = to_canonical_bytes(
    &format_path(path).to_string(),
    document,
    resolve_line_endings(line_endings, existing.as_deref()),
  )?;

  Ok(CanonicalRender { bytes, existing })
}

/// Publish a rendered source, unless it would change nothing.
///
/// Answers whether it wrote. A file already holding these exact bytes is left alone rather than replaced with itself,
/// so a run over a clean tree touches no mtimes and provokes no watcher.
///
/// # Errors
///
/// Returns an IO error when the file cannot be created or replaced.
pub(crate) fn write_canonical(path: &Path, render: &CanonicalRender) -> XrfResult<bool> {
  if !render.is_changed() {
    return Ok(false);
  }

  if render.is_new() {
    write_new_file_staged(path, &render.bytes)?;
  } else {
    write_file_staged(path, &render.bytes)?;
  }

  Ok(true)
}

/// Render and publish one source in a single step, for a caller with no use for the intermediate.
///
/// # Errors
///
/// Whatever [`render_canonical`] and [`write_canonical`] return.
pub(crate) fn write_canonical_document(
  path: &Path,
  document: &TranslationJson,
  line_endings: Option<LineEndings>,
) -> XrfResult<bool> {
  write_canonical(path, &render_canonical(path, document, line_endings)?)
}

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

  // Through the canonical writer, so an editor save and an import leave a file in the same shape. The document keeps
  // whatever order it was read in: an editor writes back what the translator sees, and reordering their file under
  // them on save is the formatter's job to be asked for, not a side effect of typing.
  write_canonical_document(path, &parsed, None)?;

  Ok(())
}
