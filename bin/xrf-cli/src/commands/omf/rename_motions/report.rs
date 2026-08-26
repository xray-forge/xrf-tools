use std::path::Path;

use serde::Serialize;
use xrf_db::OmfFile;

/// What `omf rename-motions` renamed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfRenameMotionsReport {
  destination: String,
  is_dry: bool,
  motions: Vec<String>,
  renamed: usize,
  source: String,
}

impl OmfRenameMotionsReport {
  pub fn new(source: &Path, destination: &Path, file: &OmfFile, renamed: usize, is_dry: bool) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      is_dry,
      motions: file.get_motion_names().into_iter().map(String::from).collect(),
      renamed,
      source: xrf_utils::to_portable_path_string(source),
    }
  }
}
