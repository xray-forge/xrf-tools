use std::path::Path;

use serde::Serialize;
use xrf_dds::DdsMetadata;

/// What `texture unpack-equipment-icons` sliced apart, and out of which sheet.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEquipmentUnpackReport {
  declared_mipmap_levels: Option<u32>,
  destination: String,
  format: String,
  height: u32,
  source: String,
  system_ltx: String,
  width: u32,
}

impl TextureEquipmentUnpackReport {
  pub fn new(source: &Path, system_ltx: &Path, destination: &Path, metadata: &DdsMetadata) -> Self {
    Self {
      declared_mipmap_levels: metadata.declared_mipmap_levels,
      destination: xrf_utils::to_portable_path_string(destination),
      format: metadata.get_format_label(),
      height: metadata.height,
      source: xrf_utils::to_portable_path_string(source),
      system_ltx: xrf_utils::to_portable_path_string(system_ltx),
      width: metadata.width,
    }
  }
}
