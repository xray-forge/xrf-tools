use serde::Serialize;
use xrf_dds::DdsMetadata;

/// The texture as it stands, for the row a comparison is read against.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEncodingCurrent {
  /// Format name from the file's own header, which is not always one of the five candidates.
  pub label: String,
  pub file_bytes: u64,
  pub gpu_bytes: u64,
  pub width: u32,
  pub height: u32,
  pub mipmap_levels: u32,
}

impl TextureEncodingCurrent {
  /// The texture a header describes.
  pub fn of(metadata: &DdsMetadata) -> Self {
    Self {
      label: metadata.get_format_label(),
      file_bytes: metadata.file_size,
      gpu_bytes: metadata.file_size.saturating_sub(metadata.metadata_size),
      width: metadata.width,
      height: metadata.height,
      mipmap_levels: metadata.mipmap_levels,
    }
  }
}
