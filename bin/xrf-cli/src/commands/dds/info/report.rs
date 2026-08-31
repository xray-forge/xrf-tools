use serde::Serialize;
use xrf_dds::DdsMetadata;

/// What `dds info` read out of a DDS file.
///
/// The format reaches a machine as its label rather than as a variant name, through
/// `DdsMetadata::get_format_label`: `DdsFormat` is non-exhaustive, so an unrecognised format keeps
/// its FourCC visible instead of disappearing from the report.
///
/// `declaredMipmapLevels` is what the header claims and `mipmapLevels` what the file actually
/// carries; the two disagreeing is a real property of X-Ray textures rather than a reading error,
/// so both are reported.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsInfoReport {
  bits_per_pixel: Option<u8>,
  block_size: Option<u32>,
  data_size: u64,
  declared_mipmap_levels: Option<u32>,
  depth: Option<u32>,
  file_size: u64,
  format: String,
  four_cc: Option<u32>,
  has_data_format: bool,
  height: u32,
  linear_size: Option<u32>,
  metadata_size: u64,
  minimum_mipmap_size: u32,
  mipmap_levels: u32,
  pitch: Option<u32>,
  width: u32,
}

impl DdsInfoReport {
  pub fn new(metadata: &DdsMetadata) -> Self {
    Self {
      bits_per_pixel: metadata.bits_per_pixel,
      block_size: metadata.block_size,
      data_size: metadata.data_size as u64,
      declared_mipmap_levels: metadata.declared_mipmap_levels,
      depth: metadata.depth,
      file_size: metadata.file_size,
      format: metadata.get_format_label(),
      four_cc: metadata.four_cc,
      has_data_format: metadata.has_data_format,
      height: metadata.height,
      linear_size: metadata.linear_size,
      metadata_size: metadata.metadata_size,
      minimum_mipmap_size: metadata.minimum_mipmap_size,
      mipmap_levels: metadata.mipmap_levels,
      pitch: metadata.pitch,
      width: metadata.width,
    }
  }
}
