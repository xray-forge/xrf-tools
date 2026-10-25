use serde::{Deserialize, Serialize};

/// Which layout a translations root is read with.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum TranslationProjectMode {
  /// XRF sources: multi-language JSON and language-suffixed XML side by side in one tree.
  #[default]
  Source,
  /// Shipped gamedata: `text\<language>\*.xml`, where the directory carries the language.
  Gamedata,
}

impl TranslationProjectMode {
  /// Logical prefix this layout keeps its string tables under.
  ///
  /// The same two prefixes `DialogProjectMode` resolves dialog text with, and intentionally so: a
  /// dialog's `<text>` key is looked up in exactly these files, so the two crates disagreeing about
  /// where they live would make a phrase resolve in one tool and not the other.
  pub const fn get_prefix(&self) -> &'static str {
    match self {
      Self::Source => "translations",
      Self::Gamedata => r"configs\text",
    }
  }
}
