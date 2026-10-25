use std::time::Duration;

use crate::{Finding, GamedataVerificationStatus};

/// What one check answers, which its report is built from.
pub trait GamedataCheckResult {
  /// How long the check took, where it timed itself.
  fn get_duration(&self) -> Option<Duration> {
    None
  }

  /// The check's verdict.
  fn get_status(&self) -> GamedataVerificationStatus;

  /// One line saying what the check counted, passed or not, such as `1/2 scripts valid`.
  fn get_summary(&self) -> String;

  /// What the check found wrong, each finding naming its rule and the asset it is about.
  fn get_findings(&self) -> &[Finding] {
    &[]
  }
}
