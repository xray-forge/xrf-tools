use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataParticlesVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) checked_particle_files_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_particle_files_count: u32,
}

impl GamedataCheckResult for GamedataParticlesVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_particle_files_count == 0)
  }

  fn get_summary(&self) -> String {
    format!(
      "{}/{} particle library files valid",
      self.checked_particle_files_count - self.invalid_particle_files_count,
      self.checked_particle_files_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataParticlesVerificationResult;
  use crate::GamedataCheckResult;

  #[test]
  fn describes_particle_library_failures() {
    let result = GamedataParticlesVerificationResult {
      checked_particle_files_count: 1,
      invalid_particle_files_count: 1,
      ..Default::default()
    };

    assert_eq!(result.get_summary(), "0/1 particle library files valid");
  }
}
