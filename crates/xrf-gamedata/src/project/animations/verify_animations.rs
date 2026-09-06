use std::time::Instant;

use xrf_error::XrfResult;

use crate::project::animations::hud_item_animations_verification_result::GamedataHudItemAnimationsVerificationResult;
use crate::project::animations::hud_item_animations_verifier::HudItemAnimationsVerifier;
use crate::project::animations::hud_motion_collisions_verification_result::GamedataHudMotionCollisionsVerificationResult;
use crate::project::animations::hud_motion_collisions_verifier::HudMotionCollisionsVerifier;
use crate::project::animations::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
use crate::project::animations::player_hud_animations_verifier::PlayerHudAnimationsVerifier;
use crate::project::animations::verify_animations_result::GamedataAnimationsVerificationResult;
use crate::{Finding, GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions};

impl GamedataProject {
  pub fn verify_animations(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataAnimationsVerificationResult> {
    options.job.check_cancelled()?;

    xrf_output::heading!(options.output, "Verify animations:");

    let started_at: Instant = Instant::now();

    let player_hud_animations: GamedataPlayerHudAnimationsVerificationResult =
      PlayerHudAnimationsVerifier::new(self, options).verify()?;
    let hud_item_animations: GamedataHudItemAnimationsVerificationResult =
      HudItemAnimationsVerifier::new(self, options).verify()?;
    let hud_motion_collisions: GamedataHudMotionCollisionsVerificationResult =
      HudMotionCollisionsVerifier::new(self, options).verify()?;

    let mut findings: Vec<Finding> = player_hud_animations.get_findings().to_vec();

    findings.extend(hud_item_animations.get_findings().to_vec());
    findings.extend(hud_motion_collisions.get_findings().to_vec());

    let result: GamedataAnimationsVerificationResult = GamedataAnimationsVerificationResult {
      duration: started_at.elapsed(),
      findings,
      hud_item_animations,
      hud_motion_collisions,
      player_hud_animations,
    };

    xrf_output::info!(
      options.output,
      "Verified gamedata animations in {}, {}",
      xrf_utils::format_duration(result.duration),
      result.get_failure_message()
    );

    options.job.check_cancelled()?;

    Ok(result)
  }
}
