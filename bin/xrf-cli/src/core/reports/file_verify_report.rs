use std::path::Path;

use serde::Serialize;
use xrf_report::Status;

/// The verdict a command reading one file reached.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVerifyReport {
  findings: Vec<String>,
  status: Status,
  subject: String,
}

impl FileVerifyReport {
  pub fn new(subject: &Path, findings: Vec<String>) -> Self {
    Self {
      status: Status::from_is_valid(findings.is_empty()),
      findings,
      subject: xrf_utils::to_portable_path_string(subject),
    }
  }

  pub fn passed(subject: &Path) -> Self {
    Self::new(subject, Vec::new())
  }

  pub fn failed(subject: &Path, finding: String) -> Self {
    Self::new(subject, vec![finding])
  }
}
