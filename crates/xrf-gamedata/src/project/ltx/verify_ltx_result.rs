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
    // Every finding fails the check, a section that breaks its scheme as much as an entry point that would not read.
    GamedataVerificationStatus::from_is_valid(
      self.format_result.invalid_files == 0 && self.verification_result.findings.is_empty(),
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
  use xrf_ltx::{LtxFinding, LtxProjectVerifyResult};

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
        // A finding no section owns, so the verdict cannot be reading the section count.
        verification_result: LtxProjectVerifyResult {
          findings: vec![LtxFinding::new_unreadable("configs/broken.ltx", "broken")],
          ..Default::default()
        },
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
