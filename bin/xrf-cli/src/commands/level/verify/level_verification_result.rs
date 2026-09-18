use std::time::Duration;

use xrf_report::Report;

use crate::commands::level::verify::level_verification_census::LevelVerificationCensus;

/// What `level verify` concluded.
#[derive(Debug)]
pub struct LevelVerificationResult {
  pub census: LevelVerificationCensus,
  pub duration: Duration,
  pub report: Report,
}
