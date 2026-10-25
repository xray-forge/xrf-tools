use std::time::Duration;

use xrf_report::Report;

use crate::commands::ogf::verify::ogf_verification_census::OgfVerificationCensus;

/// Outcome of a sweep: a finalized report and the counts behind it.
#[derive(Debug)]
pub struct OgfVerificationResult {
  pub census: OgfVerificationCensus,
  pub duration: Duration,
  pub report: Report,
}
