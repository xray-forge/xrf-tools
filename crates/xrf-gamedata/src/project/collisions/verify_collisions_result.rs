use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

/// What one source holds that the engine cannot reach, independently of any asset kind.
#[derive(Default)]
pub struct GamedataCollisionsVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) unreachable_files_count: usize,
  pub(crate) findings: Vec<Finding>,
}

impl GamedataCheckResult for GamedataCollisionsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  /// Reports a defect list rather than a certification, which is why a clean project answers `Skipped`.
  ///
  /// This check runs whatever `--checks` selected, so `Passed` would lift the aggregate of a run whose selected checks
  /// judged nothing: a reachable index says nothing about content. Having nothing to add to a verdict is exactly what
  /// `Skipped` means here, and the summary states what was examined.
  fn get_status(&self) -> GamedataVerificationStatus {
    if self.unreachable_files_count == 0 {
      GamedataVerificationStatus::Skipped
    } else {
      GamedataVerificationStatus::Failed
    }
  }

  fn get_failure_message(&self) -> String {
    if self.unreachable_files_count == 0 {
      String::from("No unreachable files")
    } else {
      format!(
        "{} file(s) cannot be reached, another file claims their path",
        self.unreachable_files_count
      )
    }
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
