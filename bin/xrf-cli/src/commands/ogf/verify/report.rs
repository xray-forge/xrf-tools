use serde::Serialize;
use xrf_report::Report;

use crate::commands::ogf::verify::ogf_verifier::OgfVerificationCensus;

/// What `ogf verify` found and counted: its checks as every verify command reports them, beside its census.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfVerifyReport<'a> {
  census: &'a OgfVerificationCensus,
  #[serde(flatten)]
  report: &'a Report,
}

impl<'a> OgfVerifyReport<'a> {
  /// The report of a sweep that counted `census` and found `report`.
  pub fn new(census: &'a OgfVerificationCensus, report: &'a Report) -> Self {
    Self { census, report }
  }
}
