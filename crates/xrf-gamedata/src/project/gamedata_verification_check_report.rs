use std::time::Duration;

use xrf_error::{XrfError, XrfResult};
use xrf_job::JobOutcome;
use xrf_report::{CheckId, CheckReport, Finding};

use crate::{
  GamedataCheckResult, GamedataFindingFactory, GamedataVerificationRule, GamedataVerificationStatus,
  GamedataVerificationType,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GamedataVerificationCheckReport {
  pub(super) outcome: JobOutcome,
  report: CheckReport,
  summary: String,
  verification_type: GamedataVerificationType,
}

impl GamedataVerificationCheckReport {
  pub fn get_duration(&self) -> Option<Duration> {
    self.report.duration()
  }

  pub fn get_findings(&self) -> &[Finding] {
    self.report.findings()
  }

  pub const fn get_status(&self) -> GamedataVerificationStatus {
    self.report.status()
  }

  pub fn get_summary(&self) -> &str {
    &self.summary
  }

  pub const fn get_verification_type(&self) -> GamedataVerificationType {
    self.verification_type
  }

  pub fn get_report(&self) -> &CheckReport {
    &self.report
  }

  pub(crate) fn from_check_result<T>(verification_type: GamedataVerificationType, result: XrfResult<T>) -> Self
  where
    T: GamedataCheckResult,
  {
    match result {
      Ok(result) => Self {
        outcome: JobOutcome::Completed,
        report: CheckReport::new(
          Self::check_id(verification_type),
          result.get_status(),
          result.get_duration(),
          result.get_findings().to_vec(),
        ),
        summary: result.get_summary(),
        verification_type,
      },
      Err(XrfError::Cancelled { .. }) => Self {
        outcome: JobOutcome::Cancelled,
        report: CheckReport::new(
          Self::check_id(verification_type),
          GamedataVerificationStatus::Incomplete,
          None,
          Vec::new(),
        ),
        summary: format!("Stopped checking {verification_type}; verification is incomplete"),
        verification_type,
      },
      Err(error) => Self {
        outcome: JobOutcome::Completed,
        report: CheckReport::new(
          Self::check_id(verification_type),
          GamedataVerificationStatus::Error,
          None,
          vec![GamedataFindingFactory::without_asset(
            GamedataVerificationRule::CheckExecution,
            error.to_string(),
          )],
        ),
        summary: format!("Check failed ({verification_type}): {error}"),
        verification_type,
      },
    }
  }

  fn check_id(verification_type: GamedataVerificationType) -> CheckId {
    CheckId::new(verification_type.to_string()).expect("Gamedata verification types have non-empty stable identifiers")
  }
}
