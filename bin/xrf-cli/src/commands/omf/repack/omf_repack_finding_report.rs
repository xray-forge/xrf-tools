use std::path::Path;

use serde::Serialize;

/// One file that did not come back byte identical.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfRepackFindingReport {
  message: String,
  source: String,
}

impl OmfRepackFindingReport {
  pub fn new(source: &Path, message: String) -> Self {
    Self {
      message,
      source: xrf_utils::to_portable_path_string(source),
    }
  }
}
