use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataLevelsVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) findings: Vec<Finding>,
  /// Whether any spawn asset provided a game graph to reconcile against.
  pub(crate) has_roster: bool,
  pub(crate) roster_levels_count: u32,
  pub(crate) checked_levels_count: u32,
  pub(crate) invalid_levels_count: u32,
  pub(crate) checked_references_count: u32,
  pub(crate) invalid_references_count: u32,
}

impl GamedataLevelsVerificationResult {
  /// Levels cannot be reconciled without a game graph, so the check reports nothing rather than
  /// passing vacuously.
  pub(crate) fn skipped(duration: Duration) -> Self {
    Self {
      duration,
      ..Default::default()
    }
  }
}

impl GamedataCheckResult for GamedataLevelsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    if !self.has_roster {
      GamedataVerificationStatus::Skipped
    } else {
      GamedataVerificationStatus::from_is_valid(self.findings.is_empty())
    }
  }

  fn get_failure_message(&self) -> String {
    if !self.has_roster {
      return String::from("No game graph found, level roster is unknown");
    }

    format!(
      "{}/{} level bundles valid; {}/{} level shader references valid",
      self.checked_levels_count - self.invalid_levels_count,
      self.checked_levels_count,
      self.checked_references_count - self.invalid_references_count,
      self.checked_references_count,
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::GamedataLevelsVerificationResult;
  use crate::{
    Finding, GamedataCheckResult, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule,
    GamedataVerificationStatus, GamedataVerificationType,
  };

  #[test]
  fn skips_verification_without_a_game_graph() {
    let result: GamedataLevelsVerificationResult = GamedataLevelsVerificationResult::skipped(Duration::ZERO);

    assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
    assert_eq!(
      result.get_failure_message(),
      "No game graph found, level roster is unknown"
    );
  }

  #[test]
  fn passes_when_every_implemented_rule_holds() {
    let result: GamedataLevelsVerificationResult = GamedataLevelsVerificationResult {
      has_roster: true,
      roster_levels_count: 5,
      checked_levels_count: 5,
      checked_references_count: 1204,
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
    assert_eq!(
      result.get_failure_message(),
      "5/5 level bundles valid; 1204/1204 level shader references valid"
    );
  }

  #[test]
  fn exposes_level_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::LevelsMissingBundle,
      "levels/zaton",
      "Game graph declares level [zaton] with id 108, but no level bundle exists for it",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Levels,
      Ok(GamedataLevelsVerificationResult {
        has_roster: true,
        roster_levels_count: 1,
        checked_levels_count: 0,
        findings: vec![finding.clone()],
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
    assert_eq!(
      report.get_checks()[0].get_summary(),
      "0/0 level bundles valid; 0/0 level shader references valid"
    );
  }
}
