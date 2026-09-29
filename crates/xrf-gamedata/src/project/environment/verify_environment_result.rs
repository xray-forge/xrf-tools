use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

/// Counts, duration and findings reported by the environment check.
#[derive(Default)]
pub struct GamedataEnvironmentVerificationResult {
  pub(crate) duration: Duration,
  /// Configs read, and configs the engine needed that were not there.
  pub(crate) checked_configs_count: u32,
  /// Configs with at least one finding.
  pub(crate) invalid_configs_count: u32,
  pub(crate) findings: Vec<Finding>,
}

impl GamedataCheckResult for GamedataEnvironmentVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.findings.is_empty())
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} environment configs valid",
      self.checked_configs_count - self.invalid_configs_count,
      self.checked_configs_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
