use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
/// Aggregated outcome of validating sound references in configs and XML.
pub(crate) struct GamedataSoundReferencesVerificationResult {
  pub(crate) checked_references_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_references_count: u32,
}

impl GamedataCheckResult for GamedataSoundReferencesVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_references_count == 0)
  }

  fn get_summary(&self) -> String {
    format!(
      "{}/{} sound references valid",
      self.checked_references_count - self.invalid_references_count,
      self.checked_references_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
