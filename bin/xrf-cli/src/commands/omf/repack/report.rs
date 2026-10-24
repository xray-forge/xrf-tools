use serde::Serialize;
use xrf_report::Status;

use crate::commands::omf::repack::{OmfRepackFindingReport, OmfRepackStatistics};

/// The verdict `omf repack` reached while verifying round trips.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfRepackVerifyReport {
  checked: u32,
  failed: u32,
  findings: Vec<OmfRepackFindingReport>,
  identical: u32,
  malformed: u32,
  mismatched: u32,
  status: Status,
}

impl OmfRepackVerifyReport {
  /// The report of round trips counted in `statistics`, with what each failing one found.
  pub fn new(statistics: &OmfRepackStatistics, findings: Vec<OmfRepackFindingReport>) -> Self {
    Self {
      checked: statistics.checked(),
      failed: statistics.failed(),
      findings,
      identical: statistics.identical(),
      malformed: statistics.malformed(),
      mismatched: statistics.mismatched(),
      status: statistics.status(),
    }
  }
}
