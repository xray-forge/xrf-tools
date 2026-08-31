use serde::Serialize;
use xrf_texture::{CropTextureOptions, CropTextureResult};

/// What `dds crop` cut out of a sheet, and what it wrote.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsCropReport {
  destination: String,
  height: u32,
  region_height: u32,
  region_width: u32,
  region_x: u32,
  region_y: u32,
  source: String,
  width: u32,
}

impl DdsCropReport {
  pub fn new(options: &CropTextureOptions, result: &CropTextureResult) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(&options.output_path),
      height: result.height,
      region_height: options.height,
      region_width: options.width,
      region_x: options.x,
      region_y: options.y,
      source: xrf_utils::to_portable_path_string(&options.source),
      width: result.width,
    }
  }
}
