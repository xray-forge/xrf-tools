use std::time::Duration;

use serde::Serialize;
use xrf_vfs::XrayPathCollision;

use crate::commands::gamedata::list::asset_lister::AssetListing;
use crate::commands::gamedata::list::gamedata_asset_report::GamedataAssetReport;
use crate::commands::gamedata::list::gamedata_mount_report::GamedataMountReport;
use crate::core::reports::SkippedMountReport;

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
  mounts: Vec<GamedataMountReport>,
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
      mounts: GamedataMountReport::list(&listing.mounts),
      origin: listing.origin.clone(),
      shadowed: GamedataAssetReport::list(&listing.shadowed),
      skipped: SkippedMountReport::list(&listing.skipped),
      total: listing.entries.len(),
    }
  }
}
