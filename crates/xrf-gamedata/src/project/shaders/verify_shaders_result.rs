use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataFindingFactory, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataShadersVerificationResult {
  pub(crate) duration: Duration,
  checked_scripts_count: u32,
  checked_sources_count: u32,
  findings: Vec<Finding>,
}

impl GamedataShadersVerificationResult {
  pub(crate) fn add_finding(&mut self, finding: Finding) {
    self.findings.push(finding);
  }

  pub(crate) fn increment_checked_scripts_count(&mut self) {
    self.checked_scripts_count += 1;
  }

  pub(crate) fn increment_checked_sources_count(&mut self) {
    self.checked_sources_count += 1;
  }

  pub(crate) fn sort_findings(&mut self) {
    self
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);
  }
}

impl GamedataCheckResult for GamedataShadersVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.findings.is_empty())
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{} shader scripts and {} shader sources checked, {} problems",
      self.checked_scripts_count,
      self.checked_sources_count,
      self.findings.len()
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataShadersVerificationResult;
  use crate::{
    Finding, GamedataCheckResult, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule,
    GamedataVerificationStatus, GamedataVerificationType,
  };

  #[test]
  fn exposes_renderer_shader_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::ShadersIncludeMissing,
      "shaders/r3/main.ps",
      "Shader source includes missing file 'common.h'",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Shaders,
      Ok(GamedataShadersVerificationResult {
        checked_sources_count: 1,
        findings: vec![finding.clone()],
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }

  #[test]
  fn summarizes_checked_renderer_shader_files() {
    let result: GamedataShadersVerificationResult = GamedataShadersVerificationResult {
      checked_scripts_count: 2,
      checked_sources_count: 3,
      ..Default::default()
    };

    assert_eq!(
      result.get_failure_message(),
      "2 shader scripts and 3 shader sources checked, 0 problems"
    );
  }
}
