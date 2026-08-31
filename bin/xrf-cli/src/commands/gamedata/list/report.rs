use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use xrf_vfs::{XrayAsset, XrayAssetContainer, XrayPathCollision};

use crate::commands::gamedata::list::asset_lister::AssetListing;
use crate::core::reports::SkippedMountReport;

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

  fn list(assets: &[XrayAsset]) -> Vec<Self> {
    assets.iter().map(Self::new).collect()
  }
}

/// What `gamedata list` resolved out of the roots it was given.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamedataListReport {
  /// Reported as the VFS records it, since one shape answers this wherever it is asked.
  collisions: Vec<XrayPathCollision>,
  #[serde(with = "xrf_utils::duration_ms")]
  duration: Duration,
  entries: Vec<GamedataAssetReport>,
  /// Whether shadowed entries were asked for, since an empty list otherwise cannot say which it means.
  is_shadowed_included: bool,
  mounts: Vec<String>,
  origin: String,
  shadowed: Vec<GamedataAssetReport>,
  skipped: Vec<SkippedMountReport>,
  total: usize,
}

impl GamedataListReport {
  pub fn new(listing: &AssetListing, is_shadowed_included: bool) -> Self {
    Self {
      collisions: listing.collisions.clone(),
      duration: listing.duration,
      entries: GamedataAssetReport::list(&listing.entries),
      is_shadowed_included,
      mounts: listing.mounts.clone(),
      origin: listing.origin.clone(),
      shadowed: GamedataAssetReport::list(&listing.shadowed),
      skipped: SkippedMountReport::list(&listing.skipped),
      total: listing.entries.len(),
    }
  }
}
