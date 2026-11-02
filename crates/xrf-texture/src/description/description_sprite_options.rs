use std::path::PathBuf;

use xrf_dds::ImageFormat;
use xrf_utils::HostLinkPolicy;

/// What a texture description run packs or unpacks, in either direction.
pub struct DescriptionSpriteOptions {
  /// Where progress goes, and for packing, where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub description: PathBuf,
  pub base: PathBuf,
  pub output: xrf_output::OutputOptions,
  pub output_path: PathBuf,
  pub dds_compression_format: ImageFormat,
  /// Names of the described files to process. Empty processes every file in the description.
  pub files: Vec<String>,
  pub is_strict: bool,
  /// What to do with a link already below `output_path`.
  pub link_policy: HostLinkPolicy,
}
