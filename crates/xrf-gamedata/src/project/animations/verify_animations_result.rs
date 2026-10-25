use std::time::Duration;

use xrf_anm::AnmFile;
use xrf_ppe::PpeFile;

use crate::project::animations::envelopes::envelope_animations_verification_result::GamedataEnvelopeAnimationsVerificationResult;
use crate::project::animations::hud::hud_item_animations_verification_result::GamedataHudItemAnimationsVerificationResult;
use crate::project::animations::hud::hud_motion_collisions_verification_result::GamedataHudMotionCollisionsVerificationResult;
use crate::project::animations::hud::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

pub struct GamedataAnimationsVerificationResult {
  pub(crate) duration: Duration,
  pub(crate) findings: Vec<Finding>,
  pub(crate) hud_item_animations: GamedataHudItemAnimationsVerificationResult,
  pub(crate) hud_motion_collisions: GamedataHudMotionCollisionsVerificationResult,
  pub(crate) object_motions: GamedataEnvelopeAnimationsVerificationResult<AnmFile>,
  pub(crate) player_hud_animations: GamedataPlayerHudAnimationsVerificationResult,
  pub(crate) post_process_effects: GamedataEnvelopeAnimationsVerificationResult<PpeFile>,
}

impl GamedataCheckResult for GamedataAnimationsVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::aggregate([
      self.player_hud_animations.get_status(),
      self.hud_item_animations.get_status(),
      self.hud_motion_collisions.get_status(),
      self.object_motions.get_status(),
      self.post_process_effects.get_status(),
    ])
  }

  fn get_summary(&self) -> String {
    format!(
      "{}, {}, {}, {}, {}",
      self.player_hud_animations.get_summary(),
      self.hud_item_animations.get_summary(),
      self.hud_motion_collisions.get_summary(),
      self.object_motions.get_summary(),
      self.post_process_effects.get_summary()
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::GamedataAnimationsVerificationResult;
  use crate::project::animations::envelopes::envelope_animations_verification_result::GamedataEnvelopeAnimationsVerificationResult;
  use crate::project::animations::hud::hud_item_animations_verification_result::GamedataHudItemAnimationsVerificationResult;
  use crate::project::animations::hud::hud_motion_collisions_verification_result::GamedataHudMotionCollisionsVerificationResult;
  use crate::project::animations::hud::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
  use crate::{
    Finding, GamedataFindingFactory, GamedataVerificationReport, GamedataVerificationRule, GamedataVerificationStatus,
    GamedataVerificationType,
  };

  #[test]
  fn exposes_animation_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::AnimationsPlayerHud,
      "configs/system.ltx",
      "Player HUD section [actor_hud] has invalid animations",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Animations,
      Ok(GamedataAnimationsVerificationResult {
        duration: Duration::ZERO,
        findings: vec![finding.clone()],
        hud_item_animations: GamedataHudItemAnimationsVerificationResult::default(),
        hud_motion_collisions: GamedataHudMotionCollisionsVerificationResult::default(),
        object_motions: GamedataEnvelopeAnimationsVerificationResult::default(),
        post_process_effects: GamedataEnvelopeAnimationsVerificationResult::default(),
        player_hud_animations: GamedataPlayerHudAnimationsVerificationResult {
          checked_huds_count: 1,
          findings: vec![finding.clone()],
          invalid_huds_count: 1,
        },
      }),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }

  #[test]
  fn fails_when_only_hud_item_animations_are_invalid() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::AnimationsHudItem,
      "configs/system.ltx",
      "Hud item section [wpn_ak74_hud] anm_shots=missing -> explicitly requested item motion is not found",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Animations,
      Ok(GamedataAnimationsVerificationResult {
        duration: Duration::ZERO,
        findings: vec![finding.clone()],
        hud_item_animations: GamedataHudItemAnimationsVerificationResult {
          checked_items_count: 2,
          findings: vec![finding.clone()],
          invalid_items_count: 1,
        },
        hud_motion_collisions: GamedataHudMotionCollisionsVerificationResult::default(),
        object_motions: GamedataEnvelopeAnimationsVerificationResult::default(),
        post_process_effects: GamedataEnvelopeAnimationsVerificationResult::default(),
        player_hud_animations: GamedataPlayerHudAnimationsVerificationResult {
          checked_huds_count: 1,
          findings: Vec::new(),
          invalid_huds_count: 0,
        },
      }),
    );

    assert_eq!(
      report.get_status(),
      GamedataVerificationStatus::Failed,
      "Expect invalid hud item animations alone to fail the animations check"
    );
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
