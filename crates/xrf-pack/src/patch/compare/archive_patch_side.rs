use serde::Serialize;
use xrf_vfs::XrayAssetContainer;

/// Which side of a comparison an entry was read from, and how big it was there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchSide {
  pub container: XrayAssetContainer,
  /// Unpacked payload size, from the name table for an archived entry and from metadata for a loose one.
  pub size: u64,
}
