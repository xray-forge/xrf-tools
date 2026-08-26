use std::path::Path;

use serde::Serialize;
use xrf_report::Status;

use super::statistics::RepackOmfStatistics;

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

/// The verdict `omf repack` reached while verifying round trips.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfRepackVerifyReport {
  checked: u32,
  failed: u32,
  findings: Vec<OmfRepackFindingReport>,
  identical: u32,
  mismatched: u32,
  status: Status,
}

impl OmfRepackVerifyReport {
  pub fn new(statistics: &RepackOmfStatistics, findings: Vec<OmfRepackFindingReport>) -> Self {
    Self {
      checked: statistics.checked(),
      failed: statistics.failed(),
      findings,
      identical: statistics.identical(),
      mismatched: statistics.mismatched(),
      status: Status::from_is_valid(statistics.is_valid()),
    }
  }
}
