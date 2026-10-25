use serde::Serialize;
use xrf_report::Report;

/// The verdict `omf repack --verify` reached over the round trips it ran.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfRepackVerifyReport {
  /// Files whose round trip ran: the denominator of the findings, which name every file that did not come back.
  checked: u32,
  #[serde(flatten)]
  report: Report,
}

impl OmfRepackVerifyReport {
  pub fn new(checked: u32, report: Report) -> Self {
    Self { checked, report }
  }
}
