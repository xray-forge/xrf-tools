use std::path::{Path, PathBuf};

use crate::project::mode::DialogProjectMode;

/// Paths a caller supplies in place of the ones a mode would default to.
///
/// A relative override is joined onto the project root; an absolute one is taken as it stands, so a
/// tree keeping its dialogs outside the project entirely still opens.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogProjectOverrides {
  pub dialogs: Option<PathBuf>,
  pub translations: Option<PathBuf>,
}

/// Where one opened project's two trees actually are.
///
/// Two roots rather than one, because the layouts disagree about more than file format: gamedata
/// hangs both under `configs`, while the XRF sources keep `configs/gameplay` and `translations` as
/// siblings. A single path cannot address the second, and the second is the one a developer edits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogProjectRoots {
  dialogs: PathBuf,
  translations: PathBuf,
}

impl DialogProjectRoots {
  /// Resolve both roots from a project root, its mode, and any overrides.
  pub fn resolve(root: &Path, mode: DialogProjectMode, overrides: &DialogProjectOverrides) -> Self {
    Self {
      dialogs: Self::resolve_one(
        root,
        overrides.dialogs.as_deref(),
        DialogProjectMode::DIALOGS_COMPONENTS,
      ),
      translations: Self::resolve_one(
        root,
        overrides.translations.as_deref(),
        mode.get_translations_components(),
      ),
    }
  }

  pub fn get_dialogs(&self) -> &Path {
    &self.dialogs
  }

  pub fn get_translations(&self) -> &Path {
    &self.translations
  }

  fn resolve_one(root: &Path, provided: Option<&Path>, components: &[&str]) -> PathBuf {
    match provided {
      Some(path) if path.is_absolute() => path.to_path_buf(),
      Some(path) => root.join(path),
      None => DialogProjectMode::join(root, components),
    }
  }
}
