use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataScriptsVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) invalid_scripts_count: u32,
  pub(crate) checked_scripts_count: u32,
  pub(crate) findings: Vec<Finding>,
}

impl GamedataCheckResult for GamedataScriptsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_scripts_count == 0)
  }

  fn get_summary(&self) -> String {
    format!(
      "{}/{} scripts valid",
      self.checked_scripts_count - self.invalid_scripts_count,
      self.checked_scripts_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
