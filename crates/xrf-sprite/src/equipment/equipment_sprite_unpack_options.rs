use std::path::PathBuf;

use image::RgbaImage;
use xrf_dds::ImageFormat;
use xrf_ltx::Ltx;
use xrf_utils::HostLinkPolicy;

use crate::report::SpriteListener;

pub struct EquipmentSpriteUnpackOptions {
  /// Where progress goes, and for packing, where cancellation comes from.
  pub job: xrf_job::JobHandle,
  /// What hears the run's notices as it meets them, or nothing.
  pub listener: Option<SpriteListener>,
  pub ltx: Ltx,
  pub source: RgbaImage,
  pub output_path: PathBuf,
  pub dds_compression_format: ImageFormat,
  /// What to do with a link already below `output_path`.
  pub link_policy: HostLinkPolicy,
}
