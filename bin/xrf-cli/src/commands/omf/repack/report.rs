use serde::Serialize;
use xrf_report::Status;

use super::statistics::RepackOmfStatistics;
use crate::commands::omf::repack::omf_repack_finding_report::OmfRepackFindingReport;

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
