use std::path::Path;
use std::time::Duration;

use serde::Serialize;

/// What reading one representation and writing another produced.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileConversionReport {
  destination: String,
  #[serde(with = "xrf_utils::duration_ms")]
  read_duration: Duration,
  source: String,
  #[serde(with = "xrf_utils::duration_ms")]
  write_duration: Duration,
}

impl FileConversionReport {
  pub fn new(source: &Path, destination: &Path, read_duration: Duration, write_duration: Duration) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      read_duration,
      source: xrf_utils::to_portable_path_string(source),
      write_duration,
    }
  }
}
