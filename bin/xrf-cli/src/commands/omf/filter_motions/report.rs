use std::path::Path;

use serde::Serialize;
use xrf_omf::OmfFile;

/// What `omf filter-motions` kept.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OmfFilterMotionsReport {
  destination: String,
  is_dry_run: bool,
  kept: Vec<String>,
  source: String,
  total: usize,
}

impl OmfFilterMotionsReport {
  pub(crate) fn new(source: &Path, destination: &Path, file: &OmfFile, total: usize, is_dry_run: bool) -> Self {
    Self {
      destination: xrf_utils::format_portable_path(destination),
      is_dry_run,
      kept: file.get_motion_names().into_iter().map(String::from).collect(),
      source: xrf_utils::format_portable_path(source),
      total,
    }
  }
}
