use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

/// How much of what the project declared the run could actually read.
#[derive(Default)]
pub struct GamedataCoverageVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) skipped_mounts_count: usize,
  pub(crate) findings: Vec<Finding>,
}

impl GamedataCheckResult for GamedataCoverageVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  /// `Incomplete` rather than `Failed`: nothing here judged content wrong, the run simply did not see all of it.
  ///
  /// A skipped source makes its assets look missing to some checks and uncounted by the rest, so no other verdict in the
  /// report means what it says. Full coverage answers `Skipped` for the reason the collisions check does — this reports
  /// an omission list rather than a certification, and `Passed` would lift the aggregate of a run that judged nothing.
  fn get_status(&self) -> GamedataVerificationStatus {
    if self.skipped_mounts_count == 0 {
      GamedataVerificationStatus::Skipped
    } else {
      GamedataVerificationStatus::Incomplete
    }
  }

  fn get_failure_message(&self) -> String {
    if self.skipped_mounts_count == 0 {
      String::from("Every declared source opened")
    } else {
      format!(
        "{} declared source(s) could not be opened, so no result covers them",
        self.skipped_mounts_count
      )
    }
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
