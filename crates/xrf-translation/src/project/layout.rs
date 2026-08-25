use xrf_error::XrfResult;
use xrf_vfs::{XrayLookupScope, XrayRoots, XrayScopedVfs, XrayVfs};

use crate::language::TranslationLanguage;
use crate::project::constants::MAP_DESC_DIRECTORY;
use crate::project::descriptor::TranslationProjectMode;
use crate::source_file_name::{TranslationSourceFileKind, TranslationSourceFileName};

/// Report which layout roots look like, for the open form to preselect.
///
/// Advisory only. The mode a project is opened with is whatever the caller passes, because the two
/// layouts save to different files and a heuristic must not be what decides that.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted.
pub fn detect_mode(roots: &XrayRoots) -> XrfResult<TranslationProjectMode> {
  Ok(detect_mode_in(&roots.open()?))
}

/// Report the layout of roots somebody else mounted.
///
/// Each mode is looked for under its own prefix, so the answer does not depend on which directory the
/// caller happened to name: a tree holding both is a source tree, because only a source tree has the
/// files that decide it.
pub fn detect_mode_in(vfs: &XrayVfs) -> TranslationProjectMode {
  if has_source_files(vfs) {
    return TranslationProjectMode::Source;
  }

  if has_language_directory(vfs) {
    return TranslationProjectMode::Gamedata;
  }

  TranslationProjectMode::Source
}

/// A source tree is recognised by what only it has: JSON maps and language-suffixed XML.
fn has_source_files(vfs: &XrayVfs) -> bool {
  let Ok(scope) = XrayLookupScope::all().with_prefix(TranslationProjectMode::Source.get_prefix()) else {
    return false;
  };

  vfs.scoped(&scope).list_entries().iter().any(|asset| {
    TranslationSourceFileName::parse(asset.get_logical_path().file_name()).is_some_and(|name| match name.get_kind() {
      TranslationSourceFileKind::Json => true,
      TranslationSourceFileKind::Xml => name.get_xml_language().is_some(),
    })
  })
}

/// Gamedata is recognised by a directory named for a language the engine would load.
fn has_language_directory(vfs: &XrayVfs) -> bool {
  let scope: XrayLookupScope = XrayLookupScope::all();
  let scoped: XrayScopedVfs = vfs.scoped(&scope);
  let prefix: &str = TranslationProjectMode::Gamedata.get_prefix();

  scoped.list_children(prefix).is_ok_and(|listing| {
    listing
      .directories
      .iter()
      .any(|name| !name.eq_ignore_ascii_case(MAP_DESC_DIRECTORY) && TranslationLanguage::from_str_single(name).is_ok())
  })
}
