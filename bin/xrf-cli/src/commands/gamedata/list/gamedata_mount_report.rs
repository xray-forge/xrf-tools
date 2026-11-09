use serde::Serialize;
use xrf_utils::format_portable_path;
use xrf_vfs::{XrayMount, XraySourceKind};

/// One mount a `gamedata list` searched, in search order.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GamedataMountReport {
  kind: XraySourceKind,
  label: String,
  /// The directory, or the directory holding the archive volumes.
  root: String,
}

impl GamedataMountReport {
  pub(crate) fn list(mounts: &[XrayMount]) -> Vec<Self> {
    mounts
      .iter()
      .map(|mount| Self {
        kind: mount.get_kind(),
        label: mount.get_label().to_string(),
        root: format_portable_path(mount.get_source().get_root_path()),
      })
      .collect()
  }
}
