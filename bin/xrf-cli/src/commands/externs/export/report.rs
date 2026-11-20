use std::path::Path;

use serde::Serialize;
use xrf_export::ExternFormat;

/// What `externs export` rendered: how many externs, from which declarations into which artifact, and in which format,
/// which the artifact's extension may have decided rather than an argument.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternsExportReport {
  destination: String,
  externs: usize,
  format: &'static str,
  source: String,
}

impl ExternsExportReport {
  pub(crate) fn new(source: &Path, destination: &Path, format: ExternFormat, externs: usize) -> Self {
    Self {
      destination: xrf_utils::format_portable_path(destination),
      externs,
      format: format.as_str(),
      source: xrf_utils::format_portable_path(source),
    }
  }
}
