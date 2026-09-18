use serde::Serialize;
use xrf_level::FogVolume;
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// One volumetric fog body of a level, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelFogVolume {
  /// The config its simulation settings come from, which is an LTX under `$game_config$` rather than anything the
  /// file itself holds. Absent where the body names none.
  pub profile: Option<ArchiveReference>,
  /// Bodies the simulation flows around.
  pub obstacles: usize,
}

impl ArchiveLevelFogVolume {
  /// Every body of a level, with the config each names resolved.
  pub fn of_all(source: &ArchiveDescribeSource, volumes: &[FogVolume]) -> Vec<Self> {
    volumes.iter().map(|volume| Self::of(source, volume)).collect()
  }

  /// One body, taken over what it names and what it avoids.
  fn of(source: &ArchiveDescribeSource, volume: &FogVolume) -> Self {
    Self {
      profile: (!volume.profile.is_empty())
        .then(|| ArchiveReference::resolve(source, XrayAssetType::Ltx, &volume.profile)),
      obstacles: volume.obstacles.len(),
    }
  }
}
