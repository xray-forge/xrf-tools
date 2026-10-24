use serde::Serialize;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayMount, XraySourceKind};

/// One mount a `gamedata list` searched, in search order.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamedataMountReport {
  kind: XraySourceKind,
  label: String,
  /// The directory, or the directory holding the archive volumes.
  root: String,
}

impl GamedataMountReport {
  pub fn list(mounts: &[XrayMount]) -> Vec<Self> {
    mounts
      .iter()
      .map(|mount| Self {
        kind: mount.get_kind(),
        label: mount.get_label().to_string(),
        root: to_portable_path_string(mount.get_source().get_root_path()),
      })
      .collect()
  }
}
