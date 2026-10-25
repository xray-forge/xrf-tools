use serde::Serialize;
use xrf_vfs::{XrayAssetContainer, XrayShadowedCopy};

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
