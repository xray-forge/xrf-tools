use std::path::PathBuf;

use xrf_dds::ImageFormat;
use xrf_ltx::Ltx;

use crate::report::SpriteListener;

pub struct EquipmentSpritePackOptions {
  /// Where progress goes, and for packing, where cancellation comes from.
  pub job: xrf_job::JobHandle,
  /// What hears the run's notices as it meets them, or nothing.
  pub listener: Option<SpriteListener>,
  pub ltx: Ltx,
  pub source: PathBuf,
  pub output_path: PathBuf,
  pub gamedata: Option<PathBuf>,
  pub dds_compression_format: ImageFormat,
  pub is_strict: bool,
}
