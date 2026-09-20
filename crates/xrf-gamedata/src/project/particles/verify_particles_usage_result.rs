use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataParticlesUsageVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) checked_references_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_references_count: u32,
  pub(crate) checked_spawn_files_count: u32,
  pub(crate) unreadable_spawn_files_count: u32,
  pub(crate) unparsed_custom_data_count: u32,
}

impl GamedataCheckResult for GamedataParticlesUsageVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    if self.invalid_references_count != 0 || self.unreadable_spawn_files_count != 0 {
      GamedataVerificationStatus::Failed
    } else if self.unparsed_custom_data_count != 0 {
      GamedataVerificationStatus::Incomplete
    } else {
      GamedataVerificationStatus::Passed
    }
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} particle references valid; {}/{} spawn files readable; {} custom data sections unparsed",
      self.checked_references_count - self.invalid_references_count,
      self.checked_references_count,
      self.checked_spawn_files_count - self.unreadable_spawn_files_count,
      self.checked_spawn_files_count,
      self.unparsed_custom_data_count,
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataParticlesUsageVerificationResult;
  use crate::{
    Finding, GamedataCheckResult, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule,
    GamedataVerificationStatus, GamedataVerificationType,
  };

  #[test]
  fn unreadable_spawn_files_fail_particle_usage_verification() {
    let result: GamedataParticlesUsageVerificationResult = GamedataParticlesUsageVerificationResult {
      checked_spawn_files_count: 1,
      unreadable_spawn_files_count: 1,
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(
      result.get_failure_message(),
      "0/0 particle references valid; 0/1 spawn files readable; 0 custom data sections unparsed"
    );
  }

  #[test]
  fn unparsed_spawn_custom_data_makes_particle_usage_verification_incomplete() {
    let result: GamedataParticlesUsageVerificationResult = GamedataParticlesUsageVerificationResult {
      unparsed_custom_data_count: 1,
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Incomplete);
    assert_eq!(
      result.get_failure_message(),
      "0/0 particle references valid; 0/0 spawn files readable; 1 custom data sections unparsed"
    );
  }

  #[test]
  fn exposes_particle_usage_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::ParticlesUsageReference,
      "configs/scripts/test.ltx",
      "Unknown particle reference: [sr_particle] name = missing_particle",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::ParticlesUsage,
      Ok(GamedataParticlesUsageVerificationResult {
        checked_references_count: 1,
        findings: vec![finding.clone()],
        invalid_references_count: 1,
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
