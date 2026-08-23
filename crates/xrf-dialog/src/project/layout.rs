use xrf_error::XrfResult;
use xrf_vfs::{XrayLookupScope, XrayMountMode, XrayVfs};

use crate::project::mode::DialogProjectMode;

const JSON_SUFFIX: &str = ".json";

/// Report which layout a mounted world looks like.
///
/// Advisory only, on the rule `xrf-translation` already states: the mode a project is opened with is
/// whatever the caller passes, because the two layouts read and write different files and a heuristic
/// must not be what decides that.
///
/// A `translations` prefix holding JSON is what only the XRF sources have. Everything else, including
/// a world this cannot make sense of, reads as gamedata — the mode dialog tooling targets.
pub fn detect_mode_in(vfs: &XrayVfs) -> DialogProjectMode {
  let Ok(scope) = XrayLookupScope::all().with_prefix(DialogProjectMode::Source.get_translations_prefix()) else {
    return DialogProjectMode::Gamedata;
  };

  let has_json: bool = vfs
    .scoped(&scope)
    .list_entries()
    .iter()
    .any(|asset| asset.get_logical_path().as_str().ends_with(JSON_SUFFIX));

  if has_json {
    DialogProjectMode::Source
  } else {
    DialogProjectMode::Gamedata
  }
}

/// Mount a path just to report which layout it looks like.
///
/// For an open form that has a path and no session yet. A caller already holding a world asks
/// [`detect_mode_in`] instead of mounting a second one.
///
/// # Errors
///
/// Returns an error when the path cannot be mounted at all.
pub fn detect_mode(mode: XrayMountMode, path: &std::path::Path) -> XrfResult<DialogProjectMode> {
  Ok(detect_mode_in(&XrayVfs::open(mode, path)?))
}
