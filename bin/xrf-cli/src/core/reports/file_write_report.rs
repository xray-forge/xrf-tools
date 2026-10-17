use std::path::Path;

use serde::Serialize;

/// What writing one artifact out of an input produced.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWriteReport {
  is_dry: bool,
  source: String,
  written: Vec<String>,
}

impl FileWriteReport {
  pub fn new(source: &Path, written: Vec<&Path>, is_dry: bool) -> Self {
    Self {
      is_dry,
      source: xrf_utils::to_portable_path_string(source),
      written: written.into_iter().map(xrf_utils::to_portable_path_string).collect(),
    }
  }
}
