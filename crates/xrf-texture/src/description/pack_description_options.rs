use std::path::PathBuf;

use xrf_dds::ImageFormat;

pub struct PackDescriptionOptions {
  /// Where progress goes, and for packing, where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub description: PathBuf,
  pub base: PathBuf,
  pub output: xrf_output::OutputOptions,
  pub output_path: PathBuf,
  pub dds_compression_format: ImageFormat,
  /// Names of the described files to pack. Empty packs every file in the description.
  pub files: Vec<String>,
  pub is_strict: bool,
}
