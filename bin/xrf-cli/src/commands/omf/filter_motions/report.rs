use std::path::Path;

use serde::Serialize;
use xrf_db::OmfFile;

/// What `omf filter-motions` kept.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfFilterMotionsReport {
  destination: String,
  is_dry: bool,
  kept: Vec<String>,
  source: String,
  total: usize,
}

impl OmfFilterMotionsReport {
  pub fn new(source: &Path, destination: &Path, file: &OmfFile, total: usize, is_dry: bool) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      is_dry,
      kept: file.get_motion_names().into_iter().map(String::from).collect(),
      source: xrf_utils::to_portable_path_string(source),
      total,
    }
  }
}
