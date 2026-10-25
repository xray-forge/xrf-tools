use serde::{Deserialize, Serialize};

/// What one language's build produced.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationBuildLanguageSummary {
  pub language: String,
  /// String tables written for this language, one per source.
  pub files: u32,
  /// Ids compiled into them, counting an id once per file it appears in.
  ///
  /// A missing translation still compiles - to the id itself, which is the engine's own fallback - so
  /// this counts what the tables hold rather than what was translated.
  pub entries: u32,
}
