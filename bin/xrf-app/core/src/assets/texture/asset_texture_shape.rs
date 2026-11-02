use serde::Serialize;
use xrf_dds::{DdsFile, DdsMetadata};

/// Pixel layout a DDS header declares.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTextureShape {
  pub width: u32,
  pub height: u32,
  /// Levels the file carries, one meaning no mip chain at all.
  pub mipmap_levels: u32,
  /// Format name from [`DdsMetadata::get_format_label`], so the viewer and the sweep agree on what a file is.
  pub format: String,
}

impl AssetTextureShape {
  /// The shape a DDS header declares, for a caller already holding the bytes.
  pub fn of_bytes(bytes: &[u8]) -> Option<Self> {
    DdsFile::read_metadata_from_bytes(bytes).ok().as_ref().map(Self::from)
  }
}

impl From<&DdsMetadata> for AssetTextureShape {
  fn from(metadata: &DdsMetadata) -> Self {
    Self {
      width: metadata.width,
      height: metadata.height,
      mipmap_levels: metadata.mipmap_levels,
      format: metadata.get_format_label(),
    }
  }
}
