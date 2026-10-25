use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_vfs::{XrayRoots, XraySearch};

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

  /// Mount roots just to report which layout they look like.
  ///
  /// For an open form that has roots and no session yet. A caller already holding one asks
  /// [`Self::detect_in`] instead of mounting a second.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted.
  pub fn detect(roots: &XrayRoots) -> XrfResult<Self> {
    Ok(Self::detect_in(&roots.to_search_plan()?.open()?))
  }

  /// Report which layout mounted roots look like.
  ///
  /// Advisory only, on the rule `xrf-translation` already states: the mode a project is opened with is
  /// whatever the caller passes, because the two layouts read and write different files and a heuristic
  /// must not be what decides that.
  ///
  /// A `translations` prefix holding JSON is what only the XRF sources have. Everything else, including
  /// roots this cannot make sense of, reads as gamedata — the mode dialog tooling targets.
  pub fn detect_in(search: &XraySearch) -> Self {
    let Ok(translations) = search.clone().with_prefix(Self::Source.get_translations_prefix()) else {
      return Self::Gamedata;
    };

    let has_json: bool = translations
      .list_files()
      .iter()
      .any(|asset| asset.get_logical_path().has_extension(XrayExtension::Json));

    if has_json { Self::Source } else { Self::Gamedata }
  }

  /// Logical prefix holding dialog text.
  pub const fn get_translations_prefix(&self) -> &'static str {
    match self {
      Self::Source => "translations",
      Self::Gamedata => r"configs\text",
    }
  }
}
