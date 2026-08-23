use std::fs;
use std::path::Path;

use crate::project::mode::DialogProjectMode;

const JSON_EXTENSION: &str = "json";

/// Report which layout a root looks like.
///
/// Advisory only, on the rule `xrf-translation` already states: the mode a project is opened with is
/// whatever the caller passes, because the two layouts read and write different files and a
/// heuristic must not be what decides that.
///
/// A `translations` directory holding JSON is what only the XRF sources have. Everything else,
/// including a root this cannot make sense of, reads as gamedata — the mode dialog tooling targets.
pub fn detect_mode(root: &Path) -> DialogProjectMode {
  let translations = DialogProjectMode::join(root, DialogProjectMode::Source.get_translations_components());

  let Ok(entries) = fs::read_dir(&translations) else {
    return DialogProjectMode::Gamedata;
  };

  let has_json: bool = entries.filter_map(Result::ok).any(|entry| {
    entry
      .path()
      .extension()
      .is_some_and(|extension| extension.eq_ignore_ascii_case(JSON_EXTENSION))
  });

  if has_json {
    DialogProjectMode::Source
  } else {
    DialogProjectMode::Gamedata
  }
}

/// Present a path the way the descriptor does, so a key matches whatever a caller compares it to.
pub(crate) fn normalize(path: &Path) -> String {
  path.to_string_lossy().replace('\\', "/")
}

/// A path relative to a root, or the whole path when it lies outside one.
pub(crate) fn relative(root: &Path, path: &Path) -> String {
  normalize(path.strip_prefix(root).unwrap_or(path))
}
