use serde::{Deserialize, Serialize};

/// Which layout a dialog project is read with.
///
/// `Gamedata` is the default, unlike `TranslationProjectMode`: dialog tooling is aimed at shipped
/// game data first, and the XRF sources are the opt-in. The two are otherwise the same distinction.
///
/// The mode decides only where dialog *text* sits. Both layouts keep the dialogs themselves at the
/// same place, and both are logical prefixes rather than host paths, so an installation reads the
/// same way a loose tree does.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum DialogProjectMode {
  /// Shipped gamedata: dialog text sits in `configs\text\<language>`, one file per language.
  #[default]
  Gamedata,
  /// XRF sources: dialog text sits in `translations`, one JSON file carrying every language.
  Source,
}

impl DialogProjectMode {
  /// Logical prefix holding dialog XML, in both layouts.
  pub const DIALOGS_PREFIX: &'static str = r"configs\gameplay";

  /// Logical prefix holding dialog text.
  pub const fn get_translations_prefix(&self) -> &'static str {
    match self {
      Self::Source => "translations",
      Self::Gamedata => r"configs\text",
    }
  }
}
