use serde::Serialize;
use xrf_report::Status;

/// One payload that could not be read back.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyFindingReport {
  message: String,
  name: String,
}

impl ArchiveVerifyFindingReport {
  pub fn new(name: &str, message: String) -> Self {
    Self {
      message,
      name: String::from(name),
    }
  }
}

/// The verdict `archive verify` reached.
///
/// Deposited whether the verdict passed or failed, because a failing check is exactly when the
/// findings explaining it are worth reporting. The status reuses [`Status`] so a consumer reads one
/// vocabulary for check outcomes across the CLI.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyReport {
  checked: usize,
  findings: Vec<ArchiveVerifyFindingReport>,
  status: Status,
}

impl ArchiveVerifyReport {
  pub fn new(checked: usize, findings: Vec<ArchiveVerifyFindingReport>) -> Self {
    Self {
      status: Status::from_is_valid(findings.is_empty()),
      checked,
      findings,
    }
  }
}
