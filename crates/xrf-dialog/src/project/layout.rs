use xrf_error::XrfResult;
use xrf_vfs::{XrayLookupScope, XrayVfs, XrayWorldSpec};

use crate::project::mode::DialogProjectMode;

const JSON_SUFFIX: &str = ".json";

/// Where inside a world this domain's data sits.
///
/// The other half of opening a project. A world says which trees are searched and in what order;
/// this says which logical prefixes inside them hold dialogs and dialog text. Every tool shares the
/// first question and answers the second for itself.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogProjectLayout {
  pub mode: DialogProjectMode,
  /// Stands in for the prefix the mode would default to.
  pub dialogs_prefix: Option<String>,
  pub translations_prefix: Option<String>,
}

impl DialogProjectLayout {
  pub fn new(mode: DialogProjectMode) -> Self {
    Self {
      mode,
      dialogs_prefix: None,
      translations_prefix: None,
    }
  }

  /// The prefix dialogs are read from.
  pub fn get_dialogs_prefix(&self) -> &str {
    self
      .dialogs_prefix
      .as_deref()
      .unwrap_or(DialogProjectMode::DIALOGS_PREFIX)
  }

  /// The prefix dialog text is read from.
  pub fn get_translations_prefix(&self) -> &str {
    self
      .translations_prefix
      .as_deref()
      .unwrap_or_else(|| self.mode.get_translations_prefix())
  }
}

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

/// Mount a world just to report which layout it looks like.
///
/// For an open form that has a world and no session yet. A caller already holding one asks
/// [`detect_mode_in`] instead of mounting a second.
///
/// # Errors
///
/// Returns an error when the world cannot be mounted.
pub fn detect_mode(world: &XrayWorldSpec) -> XrfResult<DialogProjectMode> {
  Ok(detect_mode_in(&world.open()?))
}
