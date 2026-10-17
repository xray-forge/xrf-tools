use std::path::Path;

use serde::Serialize;
use xrf_vfs::{XrayAsset, XrayAssetContainer};

/// One asset a mount resolved: its engine identity and the container that answered for it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamedataAssetReport {
  container: String,
  is_archived: bool,
  logical_path: String,
}

impl GamedataAssetReport {
  fn new(asset: &XrayAsset) -> Self {
    let (container, is_archived): (&Path, bool) = match asset.get_container() {
      XrayAssetContainer::Directory { root, .. } => (root.as_path(), false),
      XrayAssetContainer::Archive { path } => (path.as_path(), true),
    };

    Self {
      container: xrf_utils::to_portable_path_string(container),
      is_archived,
      logical_path: String::from(asset.get_logical_path().as_str()),
    }
  }

  pub fn list(assets: &[XrayAsset]) -> Vec<Self> {
    assets.iter().map(Self::new).collect()
  }
}
