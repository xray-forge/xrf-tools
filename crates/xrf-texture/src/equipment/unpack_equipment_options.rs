use std::path::PathBuf;

use image::RgbaImage;
use xrf_dds::ImageFormat;
use xrf_ltx::Ltx;

pub struct UnpackEquipmentOptions {
  /// Where progress goes, and for packing, where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub ltx: Ltx,
  pub source: RgbaImage,
  pub output: xrf_output::OutputOptions,
  pub output_path: PathBuf,
  pub dds_compression_format: ImageFormat,
}
