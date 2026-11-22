use std::path::Path;
use std::time::Duration;

use xrf_report::Finding;

use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_finding_factory::GamedataFindingFactory;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;

/// Counts, duration and findings reported by the dialogs check.
#[derive(Default)]
pub(crate) struct GamedataDialogsVerificationResult {
  pub(crate) duration: Duration,
  /// Whether the tree holds dialog files at all; one without them, such as a mod touching only meshes, is not judged.
  pub(crate) has_dialogs: bool,
  /// Dialog ids, each judged by the first declaration of it, which is the one the engine reads.
  pub(crate) checked_dialogs_count: u32,
  /// Dialogs the engine cannot open or stops in.
  pub(crate) invalid_dialogs_count: u32,
  pub(crate) checked_profiles_count: u32,
  /// Profiles no NPC can be spawned with.
  pub(crate) unresolved_profiles_count: u32,
  /// Dialog ids characters, info portions and logic sections name, and profiles sections name.
  pub(crate) checked_references_count: u32,
  /// Named dialogs and profiles nothing declares.
  pub(crate) missing_references_count: u32,
  /// Findings for what the engine asserts on.
  pub(crate) failing_count: u32,
  /// Findings for what the engine tolerates, or asserts on only once something nobody can tell from configs spawns it:
  /// a line shown as its key, a phrase never reached, a heal dialog that never matches, a profile no NPC can be spawned
  /// with, a file this check could not read. Reported always, gating only under strict.
  pub(crate) tolerated_count: u32,
  /// Whether the run was asked to judge what the engine tolerates, which promotes those findings to failing.
  pub(crate) is_strict: bool,
  pub(crate) findings: Vec<Finding>,
}

impl GamedataDialogsVerificationResult {
  /// Records a defect the engine asserts on.
  pub(crate) fn fail(&mut self, rule: GamedataVerificationRule, path: &Path, message: String) {
    self.failing_count += 1;
    self
      .findings
      .push(GamedataFindingFactory::for_asset(rule, path, message));
  }

  /// Records a defect the engine tolerates.
  pub(crate) fn tolerate(&mut self, rule: GamedataVerificationRule, path: &Path, message: String) {
    self.tolerated_count += 1;
    self
      .findings
      .push(GamedataFindingFactory::for_asset(rule, path, message));
  }
}

impl GamedataCheckResult for GamedataDialogsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    if !self.has_dialogs {
      return GamedataVerificationStatus::Skipped;
    }

    GamedataVerificationStatus::from_is_valid(self.failing_count == 0 && (!self.is_strict || self.tolerated_count == 0))
  }

  fn get_summary(&self) -> String {
    if !self.has_dialogs {
      return String::from("No dialog files found");
    }

    let mut message: String = format!(
      "{}/{} dialogs valid; {}/{} profiles resolve; {}/{} references declared",
      self.checked_dialogs_count - self.invalid_dialogs_count,
      self.checked_dialogs_count,
      self.checked_profiles_count - self.unresolved_profiles_count,
      self.checked_profiles_count,
      self.checked_references_count - self.missing_references_count,
      self.checked_references_count,
    );

    // Only when there is something to say: a clean tree reads as the counts alone.
    if self.tolerated_count > 0 {
      message.push_str(&format!(", {} defects the engine tolerates", self.tolerated_count));
    }

    message
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
