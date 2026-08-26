use std::path::Path;

use serde::Serialize;
use xrf_db::OmfFile;

/// What `omf duplicate-motion` copied.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfDuplicateMotionReport {
  destination: String,
  from: String,
  is_play_once: bool,
  motions: usize,
  source: String,
  to: String,
}

impl OmfDuplicateMotionReport {
  pub fn new(source: &Path, destination: &Path, file: &OmfFile, from: &str, to: &str, is_play_once: bool) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      from: String::from(from),
      is_play_once,
      motions: file.parameters.motions.len(),
      source: xrf_utils::to_portable_path_string(source),
      to: String::from(to),
    }
  }
}
