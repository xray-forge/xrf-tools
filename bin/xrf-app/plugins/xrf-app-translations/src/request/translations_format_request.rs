use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// What a translation formatting run, or a check of one, was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationsFormatRequest {
  /// Project directory to format.
  pub directory: PathBuf,
  /// Line endings to write, or nothing to keep what each file already uses.
  pub line_endings: Option<String>,
}
