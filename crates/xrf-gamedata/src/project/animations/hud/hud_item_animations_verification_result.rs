use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub(crate) struct GamedataHudItemAnimationsVerificationResult {
  pub(crate) checked_items_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_items_count: u32,
}

impl GamedataCheckResult for GamedataHudItemAnimationsVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_items_count == 0)
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} HUD item animations valid",
      self.checked_items_count - self.invalid_items_count,
      self.checked_items_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
