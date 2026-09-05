use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

/// Aggregated outcome of reading every texture file as an X-Ray DDS.
#[derive(Default)]
pub(crate) struct GamedataTextureFilesVerificationResult {
  pub(crate) checked_textures_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_textures_count: u32,
}

impl GamedataCheckResult for GamedataTextureFilesVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_textures_count == 0)
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} textures valid",
      self.checked_textures_count - self.invalid_textures_count,
      self.checked_textures_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
