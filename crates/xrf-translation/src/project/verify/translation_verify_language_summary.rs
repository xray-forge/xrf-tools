use serde::{Deserialize, Serialize};

/// What one language is missing from one file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationVerifyLanguageSummary {
  /// The source this counts, as the project addresses it.
  pub file: String,
  pub language: String,
  /// Ids the file holds, which is the same for every language of that file.
  pub checked: u32,
  /// Ids with no text for this language, counting an explicit `null` as missing.
  pub missing: u32,
}
