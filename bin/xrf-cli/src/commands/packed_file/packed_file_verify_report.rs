use std::path::Path;

use serde::Serialize;
use xrf_report::Report;

/// The verdict a packed-file `verify` reached, beside the file or folder it read.
///
/// The path is carried because the envelope names only the command: without it a saved report of a passing run would
/// not say what passed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackedFileVerifyReport {
  path: String,
  #[serde(flatten)]
  report: Report,
}

impl PackedFileVerifyReport {
  pub fn new(path: &Path, report: Report) -> Self {
    Self {
      path: xrf_utils::to_portable_path_string(path),
      report,
    }
  }
}
