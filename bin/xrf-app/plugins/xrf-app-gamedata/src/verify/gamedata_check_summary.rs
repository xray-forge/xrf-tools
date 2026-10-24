use std::time::Duration;

use serde::Serialize;
use xrf_gamedata::{GamedataVerificationCheckReport, GamedataVerificationType};
use xrf_report::Status;

/// One check's verdict, as the desktop surface shows it.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GamedataCheckSummary {
  /// The check that ran.
  pub check: GamedataVerificationType,
  pub status: Status,
  /// The check's own one-line verdict.
  pub summary: String,
  pub findings: usize,
  /// How long this check took, where it measured itself.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub duration: Option<Duration>,
}

impl GamedataCheckSummary {
  /// What one check's report says, as the desktop surface reads it.
  pub fn of(check: &GamedataVerificationCheckReport) -> Self {
    Self {
      check: check.get_verification_type(),
      status: check.get_status(),
      summary: check.get_summary().to_owned(),
      findings: check.get_findings().len(),
      duration: check.get_duration(),
    }
  }
}
