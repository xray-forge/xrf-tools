use xrf_error::{XrfError, XrfResult};
use xrf_utils::{LineEndings, natural_cmp};

use crate::text::translation_json::TranslationJson;

/// The canonical form every writer of a source shares: ids and languages in natural order, two-space indentation, a
/// trailing newline, and the line endings the file already uses unless a caller names others.
pub(crate) struct TranslationJsonCanon;

impl TranslationJsonCanon {
  /// Sort ids and language keys into the order a canonical source carries them.
  pub(crate) fn sort(document: &mut TranslationJson) {
    document.sort_by(|left, _, right, _| natural_cmp(left, right));

    for entry in document.values_mut() {
      entry.sort_by(|left, _, right, _| natural_cmp(left, right));
    }
  }

  /// Render a document as the bytes a canonical source holds.
  ///
  /// # Errors
  ///
  /// Returns a serialization error when the document cannot be rendered as JSON.
  pub(crate) fn to_bytes(subject: &str, document: &TranslationJson, line_endings: LineEndings) -> XrfResult<Vec<u8>> {
    let rendered: String = serde_json::to_string_pretty(document).map_err(|error| {
      XrfError::new_serialization_error(format!("Failed to serialize translation JSON '{subject}': {error}"))
    })?;

    // Appended rather than preserved. Two of this crate's writers used to keep whatever a file already had and a third
    // silently dropped it, which is three answers to a question that has one.
    Ok(line_endings.apply(&format!("{rendered}\n")).into_bytes())
  }

  /// The line endings a rewrite of `existing` should use: the caller's explicit choice, else the file's own, else LF.
  pub(crate) fn resolve_line_endings(requested: Option<LineEndings>, existing: Option<&[u8]>) -> LineEndings {
    requested
      .or_else(|| existing.and_then(LineEndings::detect))
      .unwrap_or(LineEndings::Lf)
  }
}
