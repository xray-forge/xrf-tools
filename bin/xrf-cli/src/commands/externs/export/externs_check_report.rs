use std::path::Path;

use serde::Serialize;
use xrf_export::ExternFormat;
use xrf_report::Report;

/// What `externs export --check` judged an artifact against the declarations, with how many externs those declare.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternsCheckReport {
  /// The artifact judged.
  destination: String,
  /// Externs the declarations declare: none where they did not parse.
  externs: usize,
  format: &'static str,
  /// The declarations root the artifact is judged against.
  source: String,
  #[serde(flatten)]
  report: Report,
}

impl ExternsCheckReport {
  pub fn new(source: &Path, destination: &Path, format: ExternFormat, externs: usize, report: Report) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      externs,
      format: format.as_str(),
      source: xrf_utils::to_portable_path_string(source),
      report,
    }
  }
}
