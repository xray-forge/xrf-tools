use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{LineEndings, StagedFile, format_path};

use crate::json::translation_json_canon::TranslationJsonCanon;
use crate::text::translation_json::TranslationJson;

/// What a canonical rewrite of one source would produce, beside what is there now.
pub(crate) struct TranslationJsonRender {
  /// The bytes a canonical source holds.
  pub bytes: Vec<u8>,
  /// What the file holds now, or nothing when it is not there yet.
  pub existing: Option<Vec<u8>>,
}

impl TranslationJsonRender {
  /// Render what one source would hold, without writing anything.
  ///
  /// `line_endings` is the caller's explicit choice; `None` preserves the file's own convention.
  ///
  /// # Errors
  ///
  /// Returns an IO error when an existing file cannot be read — which is not the same as it being absent — and a
  /// serialization error when the document cannot be rendered.
  pub(crate) fn render(path: &Path, document: &TranslationJson, line_endings: Option<LineEndings>) -> XrfResult<Self> {
    let existing: Option<Vec<u8>> = match fs::read(path) {
      Ok(bytes) => Some(bytes),
      // Absent is a state, not a failure: the importer creates sources that were never there.
      Err(error) if error.kind() == ErrorKind::NotFound => None,
      Err(error) => return Err(XrfError::from(error)),
    };

    let bytes: Vec<u8> = TranslationJsonCanon::to_bytes(
      &format_path(path).to_string(),
      document,
      TranslationJsonCanon::resolve_line_endings(line_endings, existing.as_deref()),
    )?;

    Ok(Self { bytes, existing })
  }

  /// Render and publish one source in a single step, for a caller with no use for the intermediate.
  ///
  /// # Errors
  ///
  /// Whatever [`Self::render`] and [`Self::write`] return.
  pub(crate) fn write_document(
    path: &Path,
    document: &TranslationJson,
    line_endings: Option<LineEndings>,
  ) -> XrfResult<bool> {
    Self::render(path, document, line_endings)?.write(path)
  }

  /// Whether writing this would change the file.
  pub(crate) fn is_changed(&self) -> bool {
    self.existing.as_deref() != Some(self.bytes.as_slice())
  }

  /// Whether the file is not there yet.
  pub(crate) fn is_new(&self) -> bool {
    self.existing.is_none()
  }

  /// Publish the rendered source, unless it would change nothing, answering whether it wrote.
  ///
  /// A file already holding these exact bytes is left alone rather than replaced with itself, so a run over a clean
  /// tree touches no mtimes and provokes no watcher.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the file cannot be created or replaced.
  pub(crate) fn write(&self, path: &Path) -> XrfResult<bool> {
    if !self.is_changed() {
      return Ok(false);
    }

    if self.is_new() {
      StagedFile::write_creating_directories(path, &self.bytes)?;
    } else {
      StagedFile::write(path, &self.bytes)?;
    }

    Ok(true)
  }
}
