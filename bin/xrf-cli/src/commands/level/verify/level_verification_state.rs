use std::time::Duration;

use xrf_report::{CheckId, CheckReport, Finding, Report, Status};

use crate::commands::level::verify::level_verification_census::LevelVerificationCensus;

/// What a sweep has found so far, gathered so a check takes one accumulator rather than four.
#[derive(Debug, Default)]
pub struct LevelVerificationState {
  pub census: LevelVerificationCensus,
  pub read: Vec<Finding>,
  pub ranges: Vec<Finding>,
  pub geometry: Vec<Finding>,
}

impl LevelVerificationState {
  /// Turns what was gathered into the report a caller judges, one check per list.
  pub fn into_report(self, duration: Duration) -> (LevelVerificationCensus, Report) {
    (
      self.census,
      Report::new(vec![
        Self::check("read", self.read, duration),
        Self::check("ranges", self.ranges, duration),
        Self::check("geometry", self.geometry, duration),
      ]),
    )
  }

  fn check(id: &str, findings: Vec<Finding>, duration: Duration) -> CheckReport {
    let status: Status = if findings.is_empty() {
      Status::Passed
    } else {
      Status::Failed
    };

    CheckReport::new(
      CheckId::new(id).expect("Expected a non-empty check id"),
      status,
      Some(duration),
      findings,
    )
  }
}
