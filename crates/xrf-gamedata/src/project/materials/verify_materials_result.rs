use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataMaterialsVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) findings: Vec<Finding>,
  /// Whether the library read at all; nothing else is checked without it.
  pub(crate) is_library_read: bool,
  pub(crate) materials_count: u32,
  pub(crate) pairs_count: u32,
  pub(crate) checked_references_count: u32,
  pub(crate) invalid_references_count: u32,
}

impl GamedataCheckResult for GamedataMaterialsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    if self.is_library_read && self.findings.is_empty() {
      GamedataVerificationStatus::Passed
    } else {
      GamedataVerificationStatus::Failed
    }
  }

  fn get_summary(&self) -> String {
    if !self.is_library_read {
      return String::from("Game material library unreadable");
    }

    format!(
      "{} materials, {} pairs; {}/{} material references valid",
      self.materials_count,
      self.pairs_count,
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
  use super::GamedataMaterialsVerificationResult;
  use crate::{GamedataCheckResult, GamedataFindingFactory, GamedataVerificationRule, GamedataVerificationStatus};

  #[test]
  fn fails_without_a_library_whatever_else_it_holds() {
    let result: GamedataMaterialsVerificationResult = GamedataMaterialsVerificationResult::default();

    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(result.get_summary(), "Game material library unreadable");
  }

  #[test]
  fn fails_on_a_library_defect_that_is_no_reference() {
    // A list over the engine's limit is a finding without being a reference, and still stops the game.
    let result: GamedataMaterialsVerificationResult = GamedataMaterialsVerificationResult {
      is_library_read: true,
      materials_count: 2,
      pairs_count: 1,
      checked_references_count: 2,
      findings: vec![GamedataFindingFactory::for_asset(
        GamedataVerificationRule::MaterialsList,
        "gamemtl.xr",
        "Pair 0 lists 7 step sounds",
      )],
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(
      result.get_summary(),
      "2 materials, 1 pairs; 2/2 material references valid"
    );
  }
}
