use std::time::Duration;

use xrf_report::Finding;

use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;

/// Counts, duration and findings reported by the environment check.
#[derive(Default)]
pub(crate) struct GamedataEnvironmentVerificationResult {
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

  fn get_summary(&self) -> String {
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
