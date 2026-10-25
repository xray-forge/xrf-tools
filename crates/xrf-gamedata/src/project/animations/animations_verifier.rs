use std::time::Instant;

use xrf_anm::AnmFile;
use xrf_error::XrfResult;
use xrf_ppe::PpeFile;

use crate::project::animations::envelopes::envelope_animations_verification_result::GamedataEnvelopeAnimationsVerificationResult;
use crate::project::animations::envelopes::envelope_animations_verifier::EnvelopeAnimationsVerifier;
use crate::project::animations::hud::hud_item_animations_verification_result::GamedataHudItemAnimationsVerificationResult;
use crate::project::animations::hud::hud_item_animations_verifier::HudItemAnimationsVerifier;
use crate::project::animations::hud::hud_motion_collisions_verification_result::GamedataHudMotionCollisionsVerificationResult;
use crate::project::animations::hud::hud_motion_collisions_verifier::HudMotionCollisionsVerifier;
use crate::project::animations::hud::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
use crate::project::animations::hud::player_hud_animations_verifier::PlayerHudAnimationsVerifier;
use crate::project::animations::verify_animations_result::GamedataAnimationsVerificationResult;
use crate::project::gamedata_check::GamedataCheck;
use crate::{Finding, GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions};

/// The animations check: the player HUD and the HUD items, motion name collisions, and every object motion and
/// post-process effect the game holds.
pub(crate) struct AnimationsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for AnimationsVerifier<'a> {
  type Result = GamedataAnimationsVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataAnimationsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify animations:");

    let started_at: Instant = Instant::now();

    let player_hud_animations: GamedataPlayerHudAnimationsVerificationResult =
      PlayerHudAnimationsVerifier::new(self.project, self.options).verify()?;
    let hud_item_animations: GamedataHudItemAnimationsVerificationResult =
      HudItemAnimationsVerifier::new(self.project, self.options).verify()?;
    let hud_motion_collisions: GamedataHudMotionCollisionsVerificationResult =
      HudMotionCollisionsVerifier::new(self.project, self.options).verify()?;
    let object_motions: GamedataEnvelopeAnimationsVerificationResult<AnmFile> =
      EnvelopeAnimationsVerifier::new(self.project, self.options).verify()?;
    let post_process_effects: GamedataEnvelopeAnimationsVerificationResult<PpeFile> =
      EnvelopeAnimationsVerifier::new(self.project, self.options).verify()?;

    let mut findings: Vec<Finding> = player_hud_animations.get_findings().to_vec();

    findings.extend(hud_item_animations.get_findings().to_vec());
    findings.extend(hud_motion_collisions.get_findings().to_vec());
    findings.extend(object_motions.get_findings().to_vec());
    findings.extend(post_process_effects.get_findings().to_vec());

    let result: GamedataAnimationsVerificationResult = GamedataAnimationsVerificationResult {
      duration: started_at.elapsed(),
      findings,
      hud_item_animations,
      hud_motion_collisions,
      object_motions,
      player_hud_animations,
      post_process_effects,
    };

    xrf_output::info!(
      self.options.output,
      "Verified gamedata animations in {}, {}",
      xrf_utils::format_duration(result.duration),
      result.get_summary()
    );

    self.options.job.check_cancelled()?;

    Ok(result)
  }
}
