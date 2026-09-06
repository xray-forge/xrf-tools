use std::path::PathBuf;

use xrf_dds::{DdsMipmaps, ImageFormat};
use xrf_output::OutputOptions;

pub struct CropTextureOptions {
  pub source: PathBuf,
  pub output_path: PathBuf,
  pub output: OutputOptions,
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  pub fit_width: Option<u32>,
  pub fit_height: Option<u32>,
  pub dds_compression_format: ImageFormat,
  pub dds_mipmaps: DdsMipmaps,
}
