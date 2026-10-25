use std::time::Duration;

use xrf_vfs::{XrayAsset, XrayMount, XrayPathCollision, XraySkippedMount};

/// Resolved assets and source metadata for one listing.
pub struct AssetListing {
  /// How the path was interpreted when planning its mounts.
  pub origin: String,
  /// Every mount, in search order.
  pub mounts: Vec<XrayMount>,
  /// Winning entries, one per logical path.
  pub entries: Vec<XrayAsset>,
  /// Entries shadowed by a higher-priority mount, absent unless asked for.
  pub shadowed: Vec<XrayAsset>,
  /// Files a mount holds but cannot reach, because another file in it claims their identity.
  pub collisions: Vec<XrayPathCollision>,
  /// Declared sources that could not be opened, so the listing does not cover them.
  pub skipped: Vec<XraySkippedMount>,
  /// Time spent planning, mounting, and enumerating.
  pub duration: Duration,
}
