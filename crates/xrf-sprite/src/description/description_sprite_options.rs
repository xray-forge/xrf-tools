use std::path::PathBuf;

use xrf_dds::ImageFormat;
use xrf_utils::HostLinkPolicy;

use crate::report::SpriteListener;

/// What a texture description run packs or unpacks, in either direction.
pub struct DescriptionSpriteOptions {
  /// Where progress goes, and for packing, where cancellation comes from.
  pub job: xrf_job::JobHandle,
  /// What hears the run's notices as it meets them, or nothing.
  pub listener: Option<SpriteListener>,
  pub description: PathBuf,
  pub base: PathBuf,
  pub output_path: PathBuf,
  pub dds_compression_format: ImageFormat,
  /// Names of the described files to process. Empty processes every file in the description.
  pub files: Vec<String>,
  pub is_strict: bool,
  /// What to do with a link already below `output_path`.
  pub link_policy: HostLinkPolicy,
  /// Whether unpacking writes over a sprite already under `output_path`; unforced, it refuses before cutting any.
  pub is_forced: bool,
}
