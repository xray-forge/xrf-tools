//! Aggregate result for assembled weather-cycle validation.

use std::time::Duration;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

/// Counts and duration reported by the weather-cycle check.
#[derive(Default)]
pub struct GamedataWeathersVerificationResult {
  /// Validation duration.
  pub(crate) duration: Duration,
  /// Number of direct weather-cycle files that were checked.
  pub(crate) checked_weather_files_count: u32,
  /// Per-file validation failures collected from weather-cycle checks.
  pub(crate) findings: Vec<Finding>,
  /// Number of checked weather-cycle files with at least one problem.
  pub(crate) invalid_weather_files_count: u32,
}

impl GamedataCheckResult for GamedataWeathersVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    if self.checked_weather_files_count == 0 {
      GamedataVerificationStatus::Failed
    } else {
      GamedataVerificationStatus::from_is_valid(self.invalid_weather_files_count == 0)
    }
  }

  fn get_failure_message(&self) -> String {
    if self.checked_weather_files_count == 0 {
      String::from("No weather files found")
    } else {
      format!(
        "{}/{} weather files valid",
        self.checked_weather_files_count - self.invalid_weather_files_count,
        self.checked_weather_files_count
      )
    }
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataWeathersVerificationResult;
  use crate::{
    Finding, GamedataCheckResult, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule,
    GamedataVerificationStatus, GamedataVerificationType,
  };

  #[test]
  fn parsed_and_validated_weather_files_pass() {
    let result: GamedataWeathersVerificationResult = GamedataWeathersVerificationResult {
      checked_weather_files_count: 1,
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
  }

  #[test]
  fn missing_weather_files_fail() {
    let result: GamedataWeathersVerificationResult = GamedataWeathersVerificationResult::default();

    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(result.get_failure_message(), "No weather files found");
  }

  #[test]
  fn exposes_weather_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::WeathersValidation,
      "configs/environment/weathers/test.ltx",
      "Weather [00:00:00] is missing required field [fog_color]",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Weathers,
      Ok(GamedataWeathersVerificationResult {
        checked_weather_files_count: 1,
        findings: vec![finding.clone()],
        invalid_weather_files_count: 1,
        ..Default::default()
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
