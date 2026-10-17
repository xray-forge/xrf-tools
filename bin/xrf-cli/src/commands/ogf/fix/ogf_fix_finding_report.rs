use std::path::Path;

use serde::Serialize;

/// One visual the command could not fix: unreadable, refused by the reader, or not written.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixFindingReport {
  message: String,
  source: String,
}

impl OgfFixFindingReport {
  pub fn new(source: &Path, message: String) -> Self {
    Self {
      message,
      source: xrf_utils::to_portable_path_string(source),
    }
  }
}
