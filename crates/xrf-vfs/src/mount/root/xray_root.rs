use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::XrayMountMode;

/// One place to read from, and how that place becomes mounts.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayRoot {
  /// Native host address, retained without rendering it as text.
  pub path: PathBuf,
  /// How this path becomes mounts. `Auto` unless the caller says otherwise.
  #[serde(default)]
  pub mode: XrayMountMode,
}

impl XrayRoot {
  pub fn new(path: PathBuf, mode: XrayMountMode) -> Self {
    Self { path, mode }
  }
}
