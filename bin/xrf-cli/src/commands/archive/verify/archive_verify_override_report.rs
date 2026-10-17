use serde::Serialize;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAssetContainer, XrayShadowingEntry};

/// One engine path the set answers with more than one copy, as a report states it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyOverrideReport {
  logical_path: String,
  /// Where the copy a lookup reaches sits.
  container: String,
  /// The copies behind it, in precedence order.
  hidden: Vec<String>,
}

impl ArchiveVerifyOverrideReport {
  /// Where a container sits, spelled the one way every report spells a path.
  fn to_portable_container(container: &XrayAssetContainer) -> String {
    match container {
      XrayAssetContainer::Directory { root, relative_path } => to_portable_path_string(root.join(relative_path)),
      XrayAssetContainer::Archive { path } => to_portable_path_string(path),
    }
  }
}

impl From<&XrayShadowingEntry> for ArchiveVerifyOverrideReport {
  fn from(entry: &XrayShadowingEntry) -> Self {
    Self {
      container: Self::to_portable_container(entry.get_asset().get_container()),
      hidden: entry
        .shadowed
        .iter()
        .map(|copy| Self::to_portable_container(copy.asset.get_container()))
        .collect(),
      logical_path: entry.get_logical_path().to_string(),
    }
  }
}
