use std::time::Duration;

use xrf_report::Finding;

use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;

/// Counts, duration and findings reported by the info portions check.
#[derive(Default)]
pub(crate) struct GamedataInfoPortionsVerificationResult {
  pub(crate) duration: Duration,
  /// Info portions the tree names anywhere: declared, given, taken or read.
  pub(crate) checked_info_portions_count: u32,
  /// Info portions something reads and nothing gives.
  pub(crate) read_only_count: u32,
  /// Configs that did not resolve, whose condlists went unread; the LTX check reports them.
  pub(crate) unread_configs_count: u32,
  /// Scripts that did not read or parse, whose names went unread; the scripts check reports them.
  pub(crate) unread_scripts_count: u32,
  /// Whether the tree holds scripts; one without, such as a mod relying on the game's, leaves what scripts give unknown,
  /// so nothing is judged.
  pub(crate) has_scripts: bool,
  /// Every finding is a warning: the engine tolerates any spelling, so a defect here stalls a quest without stopping
  /// the game.
  pub(crate) findings: Vec<Finding>,
}

impl GamedataCheckResult for GamedataInfoPortionsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    if self.checked_info_portions_count == 0 || !self.has_scripts {
      return GamedataVerificationStatus::Skipped;
    }

    GamedataVerificationStatus::from_findings(&self.findings)
  }

  fn get_summary(&self) -> String {
    if self.checked_info_portions_count == 0 {
      return String::from("No info portions found");
    }

    if !self.has_scripts {
      return format!(
        "{} info portions named, but no scripts, so which are never given is unknown",
        self.checked_info_portions_count
      );
    }

    let mut message: String = format!(
      "{} info portions named; {} read but never given",
      self.checked_info_portions_count, self.read_only_count
    );

    // Only when there is something to say: what went unread may hide a giver.
    if self.unread_configs_count > 0 || self.unread_scripts_count > 0 {
      message.push_str(&format!(
        "; {} configs and {} scripts unread",
        self.unread_configs_count, self.unread_scripts_count
      ));
    }

    message
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
