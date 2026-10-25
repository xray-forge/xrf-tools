use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::FsgameFile;
use crate::mount::XrayMountPlan;

/// The kind a probed path belongs to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayRootKind {
  /// The path declares an installation with an fsgame file.
  Installation,
  /// The path is one archive volume, or a directory of them.
  Volumes,
  /// The path is a directory holding content an engine would load.
  Root,
  /// The path is a directory, but nothing beneath it looks like game data.
  Unrecognized,
  /// Nothing is there, or it cannot be read.
  Missing,
}

impl XrayRootKind {
  /// What a path is, before anything is opened or judged by its contents.
  pub(crate) fn of(path: &Path) -> Self {
    if FsgameFile::find(path).is_some() {
      Self::Installation
    } else if XrayMountPlan::is_volume(path) || XrayMountPlan::holds_volumes(path) {
      Self::Volumes
    } else {
      Self::Root
    }
  }

  /// The plan this kind means for `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when a declared fsgame file cannot be read, decoded, or parsed.
  pub(crate) fn plan(self, path: &Path) -> XrfResult<XrayMountPlan> {
    match self {
      Self::Installation => XrayMountPlan::from_fsgame(path),
      Self::Volumes => XrayMountPlan::volumes(path),
      // A directory nothing identifies is still mounted as the root it was named as. The two kinds only a probe
      // reports mean the same thing here, because each still describes a path a caller may ask to mount.
      Self::Root | Self::Unrecognized | Self::Missing => XrayMountPlan::root(path),
    }
  }
}
