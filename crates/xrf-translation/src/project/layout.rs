use xrf_error::XrfResult;
use xrf_vfs::{XrayRoots, XraySearch};

use crate::language::TranslationLanguage;
use crate::project::constants::MAP_DESC_DIRECTORY;
use crate::project::descriptor::TranslationProjectMode;
use crate::source_file_name::is_json_source_name;

/// Report which layout roots look like, for the open form to preselect.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted.
pub fn detect_mode(roots: &XrayRoots) -> XrfResult<TranslationProjectMode> {
  Ok(detect_mode_in(&roots.to_search_plan()?.open()?))
}

/// Report the layout of roots somebody else mounted.
pub fn detect_mode_in(search: &XraySearch) -> TranslationProjectMode {
  if has_source_files(search) {
    return TranslationProjectMode::Source;
  }

  if has_language_directory(search) {
    return TranslationProjectMode::Gamedata;
  }

  TranslationProjectMode::Source
}

/// A source tree is recognised by what only it has: multi-language JSON maps.
fn has_source_files(search: &XraySearch) -> bool {
  let Ok(sources) = search.clone().with_prefix(TranslationProjectMode::Source.get_prefix()) else {
    return false;
  };

  sources
    .list_files()
    .iter()
    .any(|asset| is_json_source_name(asset.get_logical_path().file_name()))
}

/// Gamedata is recognised by a directory named for a language the engine would load.
fn has_language_directory(search: &XraySearch) -> bool {
  let prefix: &str = TranslationProjectMode::Gamedata.get_prefix();

  search.list_children(prefix).is_ok_and(|listing| {
    listing
      .directories
      .iter()
      .any(|name| !name.eq_ignore_ascii_case(MAP_DESC_DIRECTORY) && TranslationLanguage::from_str_single(name).is_ok())
  })
}
