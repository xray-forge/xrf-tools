use serde::Serialize;
use xrf_vfs::{XrayAsset, XrayAssetContainer, XrayMountedEntry};

/// One file of a mounted world, as the explorer lists it.
///
/// Shaped like [`xrf_archive::ArchiveFileDescriptor`] where the two can agree — a `name` and a `size_real` — because
/// the tree, the filter and the preview gate above them need nothing else, and giving each subject its own spelling of
/// those two would fork every one of those surfaces.
///
/// Where they cannot agree, this says less rather than inventing something. A loose file has no volume position, no
/// stored size and no recorded CRC, so nothing here claims one; what it has instead is the copies it stands in front
/// of, which a volume set has no way to express.
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
  pub shadowed: Vec<XrayAssetContainer>,
}

impl From<XrayMountedEntry> for ArchiveWorldEntry {
  fn from(entry: XrayMountedEntry) -> Self {
    Self {
      name: entry.asset.get_logical_path().as_str().to_string(),
      size_real: entry.size,
      container: entry.asset.into_container(),
      shadowed: entry.shadowed.into_iter().map(XrayAsset::into_container).collect(),
    }
  }
}
