use std::path::PathBuf;

use xrf_dds::{DdsMipmaps, ImageFormat};

pub struct CropTextureOptions {
  pub source: PathBuf,
  pub output_path: PathBuf,
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  pub fit_width: Option<u32>,
  pub fit_height: Option<u32>,
  pub dds_compression_format: ImageFormat,
  pub dds_mipmaps: DdsMipmaps,
}

impl CropTextureOptions {
  /// The bounds the cropped region is fitted into, when both are given; with either missing it is written as cut.
  pub fn get_fit(&self) -> Option<(u32, u32)> {
    self.fit_width.zip(self.fit_height)
  }
}
