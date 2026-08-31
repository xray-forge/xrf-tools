use std::time::Duration;

use xrf_error::XrfResult;
use xrf_report::{CheckId, CheckReport, Finding, Report};

use crate::{
  GamedataCheckResult, GamedataFindingFactory, GamedataVerificationRule, GamedataVerificationStatus,
  GamedataVerificationType,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GamedataVerificationCheckReport {
  report: CheckReport,
  summary: String,
  verification_type: GamedataVerificationType,
}

#[derive(Default)]
pub struct GamedataVerificationReport {
  checks: Vec<GamedataVerificationCheckReport>,
  duration: Duration,
  outcome: xrf_job::JobOutcome,
}

pub type GamedataVerificationResult = GamedataVerificationReport;

impl GamedataVerificationReport {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_duration(duration: Duration) -> Self {
    Self {
      duration,
      ..Self::default()
    }
  }

  pub fn get_checks(&self) -> &[GamedataVerificationCheckReport] {
    &self.checks
  }

  /// Whether every selected check ran, or the run was stopped between them.
  ///
  /// What separates a clean verdict from a partial one: the checks that never ran report nothing, and nothing about
  /// their silence says they would have passed.
  pub const fn get_outcome(&self) -> xrf_job::JobOutcome {
    self.outcome
  }

  pub(crate) fn set_outcome(&mut self, outcome: xrf_job::JobOutcome) {
    self.outcome = outcome;
  }

  pub const fn get_duration(&self) -> Duration {
    self.duration
  }

  pub(crate) fn set_duration(&mut self, duration: Duration) {
    self.duration = duration;
  }

  pub(crate) fn add_report(&mut self, report: GamedataVerificationCheckReport) {
    self.checks.push(report);
  }

  pub fn add_check<T>(&mut self, verification_type: GamedataVerificationType, result: XrfResult<T>)
  where
    T: GamedataCheckResult,
  {
    self.checks.push(GamedataVerificationCheckReport::from_check_result(
      verification_type,
      result,
    ));
  }

  pub fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::aggregate(self.checks.iter().map(|it| it.get_status()))
  }

  pub fn is_valid(&self) -> bool {
    self.get_status() == GamedataVerificationStatus::Passed
  }

  pub fn get_failure_messages(&self) -> Vec<String> {
    self.get_failure_reports().map(|it| it.summary.clone()).collect()
  }

  pub fn get_failure_reports(&self) -> impl Iterator<Item = &GamedataVerificationCheckReport> {
    self.checks.iter().filter(|it| {
      !matches!(
        it.get_status(),
        GamedataVerificationStatus::Passed | GamedataVerificationStatus::Skipped
      )
    })
  }

  /// Creates a shared report without gamedata-specific summaries.
  ///
  /// Gamedata retains its check counters and human summaries separately.
  pub fn to_report(&self) -> Report {
    let checks: Vec<CheckReport> = self.checks.iter().map(|check| check.report.clone()).collect();

    Report::new(checks)
  }
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
        report: CheckReport::new(
          Self::check_id(verification_type),
          result.get_status(),
          result.get_duration(),
          result.get_findings().to_vec(),
        ),
        summary: result.get_failure_message(),
        verification_type,
      },
      Err(error) => Self {
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

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;
  use xrf_report::Status;

  use super::GamedataVerificationReport;
  use crate::{
    Finding, GamedataCheckResult, GamedataFindingFactory, GamedataVerificationRule, GamedataVerificationStatus,
    GamedataVerificationType,
  };

  struct TestCheckResult {
    findings: Vec<Finding>,
    status: GamedataVerificationStatus,
    summary: String,
  }

  impl GamedataCheckResult for TestCheckResult {
    fn get_status(&self) -> GamedataVerificationStatus {
      self.status
    }

    fn get_failure_message(&self) -> String {
      self.summary.clone()
    }

    fn get_findings(&self) -> &[Finding] {
      &self.findings
    }
  }

  #[test]
  fn empty_verification_result_is_skipped_and_not_valid() {
    let result = GamedataVerificationReport::default();

    assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
    assert!(!result.is_valid());
  }

  #[test]
  fn collects_check_summaries_and_findings() {
    let mut result = GamedataVerificationReport::default();

    result.add_check(
      GamedataVerificationType::Scripts,
      Ok(TestCheckResult {
        findings: vec![GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ScriptsSyntax,
          "scripts/invalid.script",
          "Expected expression after '='",
        )],
        status: GamedataVerificationStatus::Failed,
        summary: String::from("1/1 scripts are invalid"),
      }),
    );

    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(
      result.get_failure_messages(),
      vec![String::from("1/1 scripts are invalid")]
    );
    assert_eq!(
      result.get_checks()[0].get_findings(),
      vec![GamedataFindingFactory::for_asset(
        GamedataVerificationRule::ScriptsSyntax,
        "scripts/invalid.script",
        "Expected expression after '='",
      )]
    );
  }

  #[test]
  fn records_checker_errors_as_findings() {
    let mut result = GamedataVerificationReport::default();

    result.add_check::<TestCheckResult>(
      GamedataVerificationType::Animations,
      Err(XrfError::new_unexpected_error("boom")),
    );

    assert_eq!(result.get_status(), GamedataVerificationStatus::Error);
    assert_eq!(
      result.get_failure_messages(),
      vec![String::from("Check failed (animations): Unexpected error: boom")]
    );
    assert_eq!(
      result.get_checks()[0].get_findings(),
      vec![GamedataFindingFactory::without_asset(
        GamedataVerificationRule::CheckExecution,
        "Unexpected error: boom",
      )]
    );

    let report = result.to_report();

    assert_eq!(report.checks()[0].duration(), None);
  }

  #[test]
  fn finalizes_shared_findings_in_the_report_model() {
    let mut result: GamedataVerificationReport = GamedataVerificationReport::default();

    result.add_check(
      GamedataVerificationType::Scripts,
      Ok(TestCheckResult {
        findings: vec![
          GamedataFindingFactory::for_asset(GamedataVerificationRule::ScriptsSyntax, "scripts/z.script", "same"),
          GamedataFindingFactory::for_asset(GamedataVerificationRule::ScriptsPath, "scripts/a.script", "same"),
          GamedataFindingFactory::for_asset(GamedataVerificationRule::ScriptsSyntax, "scripts/a.script", "same"),
        ],
        status: GamedataVerificationStatus::Failed,
        summary: String::from("3 scripts are invalid"),
      }),
    );

    let report = result.to_report();

    assert_eq!(report.status(), Status::Failed);
    assert_eq!(report.checks()[0].id().as_str(), "scripts");
    assert_eq!(report.checks()[0].findings()[0].rule_id().as_str(), "scripts.path");
    assert_eq!(report.checks()[0].findings()[1].rule_id().as_str(), "scripts.syntax");
    assert_eq!(report.checks()[0].findings()[2].subject(), Some("scripts/z.script"));
  }
}
