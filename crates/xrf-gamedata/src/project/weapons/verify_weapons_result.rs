use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataWeaponVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) checked_weapons_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_weapons_count: u32,
}

impl GamedataCheckResult for GamedataWeaponVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_weapons_count == 0)
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} weapons valid",
      self.checked_weapons_count - self.invalid_weapons_count,
      self.checked_weapons_count
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataWeaponVerificationResult;
  use crate::{
    Finding, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule, GamedataVerificationStatus,
    GamedataVerificationType,
  };

  #[test]
  fn exposes_weapon_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::WeaponsValidation,
      "configs/system.ltx",
      "Weapon section [wpn_test] is invalid",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Weapons,
      Ok(GamedataWeaponVerificationResult {
        checked_weapons_count: 1,
        findings: vec![finding.clone()],
        invalid_weapons_count: 1,
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
