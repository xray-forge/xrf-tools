use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Which layout a dialog project is read with.
///
/// `Gamedata` is the default, unlike `TranslationProjectMode`: dialog tooling is aimed at shipped
/// game data first, and the XRF sources are the opt-in. The two are otherwise the same distinction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum DialogProjectMode {
  /// Shipped gamedata: dialog text sits in `configs/text/<language>`, one file per language.
  #[default]
  Gamedata,
  /// XRF sources: dialog text sits in `translations`, one JSON file carrying every language.
  Source,
}

impl DialogProjectMode {
  /// Where dialog XML sits, relative to the project root.
  ///
  /// The one thing the two layouts agree on, which is why the modes differ only in their text root.
  pub const DIALOGS_COMPONENTS: &'static [&'static str] = &["configs", "gameplay"];

  /// Where dialog text sits, relative to the project root.
  pub const fn get_translations_components(&self) -> &'static [&'static str] {
    match self {
      Self::Source => &["translations"],
      Self::Gamedata => &["configs", "text"],
    }
  }

  /// Join components onto a root.
  ///
  /// Built from components rather than written as a separator-joined literal, because such a literal
  /// is one path component on Linux and several on Windows.
  pub fn join(root: &Path, components: &[&str]) -> PathBuf {
    components
      .iter()
      .fold(root.to_path_buf(), |path, component| path.join(component))
  }
}
