use serde::Serialize;
use xrf_archive_stats::{ArchiveStatisticsEntry, ArchiveWorldStatisticsEntry};
use xrf_vfs::{XrayAsset, XrayAssetContainer, XrayShadowedCopy, XrayShadowingEntry};

/// One copy of an engine path no lookup reaches, as the explorer lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShadowedCopy {
  /// Where this copy physically sits — a file on disk, or an entry of a volume.
  pub container: XrayAssetContainer,
  /// Payload bytes once unpacked, as the mount holding this copy records or measures them.
  pub size_real: u64,
}

impl From<XrayShadowedCopy> for ArchiveShadowedCopy {
  fn from(copy: XrayShadowedCopy) -> Self {
    Self {
      container: copy.asset.into_container(),
      size_real: copy.size,
    }
  }
}

/// One file of a mounted world, as the explorer lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveWorldEntry {
  /// Engine identity: lower-case and backslash separated, which is what every read of this world is addressed by.
  pub name: String,
  /// Where the winning copy physically sits — a file on disk, or an entry of a volume.
  pub container: XrayAssetContainer,
  /// Payload bytes once unpacked.
  pub size_real: u64,
  /// Copies of this engine path no lookup reaches, in mount priority order behind the winner.
  pub shadowed: Vec<ArchiveShadowedCopy>,
}

/// Measured as it already sits, rather than converted into a shape a breakdown owns.
impl ArchiveStatisticsEntry for ArchiveWorldEntry {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn get_size_real(&self) -> u64 {
    self.size_real
  }
}

impl ArchiveWorldStatisticsEntry for ArchiveWorldEntry {
  fn get_container(&self) -> &XrayAssetContainer {
    &self.container
  }

  fn list_shadowed(&self) -> impl Iterator<Item = (&XrayAssetContainer, u64)> {
    self.shadowed.iter().map(|copy| (&copy.container, copy.size_real))
  }
}

impl From<XrayShadowingEntry> for ArchiveWorldEntry {
  fn from(entry: XrayShadowingEntry) -> Self {
    let XrayShadowingEntry { entry, shadowed } = entry;

    Self {
      name: entry.asset.get_logical_path().as_str().to_string(),
      size_real: entry.size,
      container: XrayAsset::into_container(entry.asset),
      shadowed: shadowed.into_iter().map(ArchiveShadowedCopy::from).collect(),
    }
  }
}
