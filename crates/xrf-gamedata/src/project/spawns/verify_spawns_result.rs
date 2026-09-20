use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataSpawnsVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) findings: Vec<Finding>,
  pub(crate) total_spawns: u32,
  pub(crate) invalid_spawns: u32,
}

impl GamedataCheckResult for GamedataSpawnsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_spawns == 0)
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} spawns valid",
      self.total_spawns - self.invalid_spawns,
      self.total_spawns
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataSpawnsVerificationResult;
  use crate::{
    Finding, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule, GamedataVerificationStatus,
    GamedataVerificationType,
  };

  #[test]
  fn exposes_spawn_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::SpawnsRead,
      "spawns/test.spawn",
      "Failed to read spawn file: invalid header",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Spawns,
      Ok(GamedataSpawnsVerificationResult {
        findings: vec![finding.clone()],
        invalid_spawns: 1,
        total_spawns: 1,
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
