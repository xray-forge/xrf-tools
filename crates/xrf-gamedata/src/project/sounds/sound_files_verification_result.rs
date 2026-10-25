use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
/// Aggregated outcome of validating individual sound files.
pub(crate) struct GamedataSoundFilesVerificationResult {
  pub(crate) checked_sounds_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_sounds_count: u32,
}

impl GamedataCheckResult for GamedataSoundFilesVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_sounds_count == 0)
  }

  fn get_summary(&self) -> String {
    format!(
      "{}/{} sounds valid",
      self.checked_sounds_count - self.invalid_sounds_count,
      self.checked_sounds_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
