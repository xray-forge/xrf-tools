use std::path::PathBuf;

use crate::project::mode::DialogProjectMode;

/// What to open, and where inside it to look.
///
/// Prefixes are engine identities, not host paths: the same two work against a loose tree and against
/// an installation whose configs live inside `db\configs`. An override stands in for one of them, so a
/// mod keeping its dialogs somewhere the layout does not predict still opens.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogProjectOptions {
  pub root: PathBuf,
  pub mode: DialogProjectMode,
  pub dialogs_prefix: Option<String>,
  pub translations_prefix: Option<String>,
}

impl DialogProjectOptions {
  pub fn new(root: impl Into<PathBuf>, mode: DialogProjectMode) -> Self {
    Self {
      root: root.into(),
      mode,
      dialogs_prefix: None,
      translations_prefix: None,
    }
  }

  /// The dialogs prefix this open will use.
  pub fn get_dialogs_prefix(&self) -> &str {
    self
      .dialogs_prefix
      .as_deref()
      .unwrap_or(DialogProjectMode::DIALOGS_PREFIX)
  }

  /// The translations prefix this open will use.
  pub fn get_translations_prefix(&self) -> &str {
    self
      .translations_prefix
      .as_deref()
      .unwrap_or_else(|| self.mode.get_translations_prefix())
  }
}
