use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub(crate) struct GamedataPlayerHudAnimationsVerificationResult {
  pub(crate) checked_huds_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_huds_count: u32,
}

impl GamedataCheckResult for GamedataPlayerHudAnimationsVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_huds_count == 0)
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} HUD animations valid",
      self.checked_huds_count - self.invalid_huds_count,
      self.checked_huds_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
