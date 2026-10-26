use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_vfs::{XrayRoots, XraySearch};

use crate::json::translation_source_name::TranslationSourceName;
use crate::language::translation_language::TranslationLanguage;
use crate::project::describe::translation_gamedata_reader::TranslationGamedataReader;

/// Which layout a translations root is read with.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum TranslationProjectMode {
  /// XRF sources: multi-language JSON files, one per string table.
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

  /// The layout roots look like, for the open form to preselect; the caller still chooses it.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted.
  pub fn detect(roots: &XrayRoots) -> XrfResult<Self> {
    Ok(Self::detect_in(&roots.to_search_plan()?.open()?))
  }

  /// The layout of roots somebody else mounted.
  pub fn detect_in(search: &XraySearch) -> Self {
    if Self::has_source_files(search) {
      return Self::Source;
    }

    if Self::has_language_directory(search) {
      return Self::Gamedata;
    }

    Self::Source
  }

  /// A source tree is recognised by what only it has: multi-language JSON maps.
  fn has_source_files(search: &XraySearch) -> bool {
    let Ok(sources) = search.clone().with_prefix(Self::Source.get_prefix()) else {
      return false;
    };

    sources
      .list_files()
      .iter()
      .any(|asset| TranslationSourceName::is_name(asset.get_logical_path().file_name()))
  }

  /// Gamedata is recognised by a directory named for a language the engine would load.
  fn has_language_directory(search: &XraySearch) -> bool {
    search.list_children(Self::Gamedata.get_prefix()).is_ok_and(|listing| {
      listing.directories.iter().any(|name| {
        !name.eq_ignore_ascii_case(TranslationGamedataReader::MAP_DESC_DIRECTORY)
          && TranslationLanguage::from_str_single(name).is_ok()
      })
    })
  }
}
