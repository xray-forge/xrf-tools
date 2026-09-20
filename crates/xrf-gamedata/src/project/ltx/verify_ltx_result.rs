use std::time::Duration;

use xrf_ltx::{LtxProjectFormatResult, LtxProjectVerifyResult};

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

#[derive(Default)]
pub struct GamedataLtxVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) findings: Vec<Finding>,
  pub(crate) format_result: LtxProjectFormatResult,
  pub(crate) verification_result: LtxProjectVerifyResult,
}

impl GamedataCheckResult for GamedataLtxVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(
      self.format_result.invalid_files == 0 && self.verification_result.invalid_sections == 0,
    )
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} LTX files formatted; {}/{} sections valid",
      self.format_result.valid_files,
      self.format_result.total_files,
      self.verification_result.valid_sections,
      self.verification_result.total_sections
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use xrf_ltx::LtxProjectVerifyResult;

  use super::GamedataLtxVerificationResult;
  use crate::{
    Finding, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule, GamedataVerificationStatus,
    GamedataVerificationType,
  };

  #[test]
  fn exposes_ltx_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::LtxFormatting,
      "configs/system.ltx",
      "LTX file needs formatting",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Ltx,
      Ok(GamedataLtxVerificationResult {
        findings: vec![finding.clone()],
        verification_result: LtxProjectVerifyResult {
          invalid_sections: 1,
          ..Default::default()
        },
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
