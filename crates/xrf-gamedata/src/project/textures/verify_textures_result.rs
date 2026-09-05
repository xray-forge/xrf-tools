use std::time::Duration;

use crate::GamedataFindingFactory;
use crate::project::textures::texture_bumps_verification_result::GamedataTextureBumpsVerificationResult;
use crate::project::textures::texture_files_verification_result::GamedataTextureFilesVerificationResult;
use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

pub struct GamedataTexturesVerificationResult {
  pub(crate) duration: Duration,
  findings: Vec<Finding>,
  pub(crate) texture_files: GamedataTextureFilesVerificationResult,
  pub(crate) texture_bumps: GamedataTextureBumpsVerificationResult,
}

impl GamedataTexturesVerificationResult {
  pub(crate) fn new(
    duration: Duration,
    texture_files: GamedataTextureFilesVerificationResult,
    texture_bumps: GamedataTextureBumpsVerificationResult,
  ) -> Self {
    let mut findings: Vec<Finding> = texture_files
      .get_findings()
      .iter()
      .chain(texture_bumps.get_findings())
      .cloned()
      .collect();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);

    Self {
      duration,
      findings,
      texture_files,
      texture_bumps,
    }
  }
}

impl GamedataCheckResult for GamedataTexturesVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::aggregate([self.texture_files.get_status(), self.texture_bumps.get_status()])
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}; {}",
      self.texture_files.get_failure_message(),
      self.texture_bumps.get_failure_message()
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::GamedataTexturesVerificationResult;
  use crate::GamedataFindingFactory;
  use crate::project::textures::texture_bumps_verification_result::GamedataTextureBumpsVerificationResult;
  use crate::project::textures::texture_files_verification_result::GamedataTextureFilesVerificationResult;
  use crate::{
    Finding, GamedataCheckResult, GamedataVerificationReport, GamedataVerificationRule, GamedataVerificationStatus,
    GamedataVerificationType,
  };

  #[test]
  fn exposes_texture_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::TexturesValidation,
      "textures/test.dds",
      "Texture uses an unsupported format",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Textures,
      Ok(GamedataTexturesVerificationResult::new(
        Duration::ZERO,
        GamedataTextureFilesVerificationResult {
          checked_textures_count: 1,
          findings: vec![finding.clone()],
          invalid_textures_count: 1,
        },
        GamedataTextureBumpsVerificationResult::default(),
      )),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }

  #[test]
  fn reads_as_both_halves_in_one_line() {
    let result: GamedataTexturesVerificationResult = GamedataTexturesVerificationResult::new(
      Duration::ZERO,
      GamedataTextureFilesVerificationResult {
        checked_textures_count: 2,
        ..Default::default()
      },
      GamedataTextureBumpsVerificationResult {
        checked_bumps_count: 1,
        ..Default::default()
      },
    );

    assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
    assert_eq!(
      result.get_failure_message(),
      "2/2 textures valid; 1/1 declared bumps resolved"
    );
  }
}
