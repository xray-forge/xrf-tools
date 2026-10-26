use std::path::Path;

use xrf_error::XrfResult;

use crate::json::translation_json_canon::TranslationJsonCanon;
use crate::json::translation_json_reader::TranslationJsonReader;
use crate::json::translation_json_render::TranslationJsonRender;
use crate::text::translation_edit::TranslationEdit;
use crate::text::translation_json::TranslationJson;

/// Applies edits to one language inside a multi-language source.
pub(crate) struct TranslationJsonWriter;

impl TranslationJsonWriter {
  /// Apply edits to one language of a source on disk.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when the file cannot be read or re-serialized, and an IO error when it cannot be replaced.
  pub(crate) fn apply_edits(path: &Path, language: &str, edits: &[TranslationEdit]) -> XrfResult {
    let mut parsed: TranslationJson = TranslationJsonReader::read(path)?;

    Self::edit(&mut parsed, language, edits);

    // Through the canonical writer, so an editor save and an import leave a file in the same shape. The document keeps
    // whatever order it was read in: an editor writes back what the translator sees, and reordering their file under
    // them on save is the formatter's job to be asked for, not a side effect of typing.
    TranslationJsonRender::write_document(path, &parsed, None)?;

    Ok(())
  }

  /// The bytes a source holding `existing` holds once one language's edits are applied, in the canonical shape
  /// [`Self::apply_edits`] writes and with the line endings `existing` uses.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when `existing` is not a translation source, and a serialization error when the edited
  /// document cannot be rendered.
  pub(crate) fn render_edits(
    subject: &str,
    existing: &[u8],
    language: &str,
    edits: &[TranslationEdit],
  ) -> XrfResult<Vec<u8>> {
    let mut parsed: TranslationJson = TranslationJsonReader::parse(subject, existing)?;

    Self::edit(&mut parsed, language, edits);

    TranslationJsonCanon::to_bytes(
      subject,
      &parsed,
      TranslationJsonCanon::resolve_line_endings(None, Some(existing)),
    )
  }

  fn edit(parsed: &mut TranslationJson, language: &str, edits: &[TranslationEdit]) {
    for edit in edits {
      match edit {
        TranslationEdit::Set { id, value } => {
          // Stored as given: a multi-line value keeps its array form here, which is the only place that form exists.
          parsed
            .entry(id.clone())
            .or_default()
            .insert(language.to_owned(), Some(value.clone()));
        }
        TranslationEdit::Remove { id } => {
          // One language, matching what removing an entry from an XML file does. Dropping every language at once is a
          // separate action the caller has to ask for by name.
          if let Some(entry) = parsed.get_mut(id) {
            entry.shift_remove(language);

            if entry.is_empty() {
              parsed.shift_remove(id);
            }
          }
        }
      }
    }
  }
}
