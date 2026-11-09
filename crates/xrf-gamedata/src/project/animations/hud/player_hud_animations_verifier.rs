use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxResolution, Section};

use crate::project::animations::hud::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
use crate::project::animations::hud::player_hud_motion_requests::PlayerHudMotionRequests;
use crate::project::animations::hud::player_hud_motion_verdict::PlayerHudMotionVerdict;
use crate::project::animations::hud::player_hud_motions::PlayerHudMotions;
use crate::project::gamedata_item_section::GamedataItemSection;
use crate::project::gamedata_visual_motions::GamedataVisualMotions;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Verifies the player hands: their visual plays motions, and every motion a weapon's HUD section asks of them.
pub(crate) struct PlayerHudAnimationsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> PlayerHudAnimationsVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataPlayerHudAnimationsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::verbose!(self.options.output, "Verify player hud animations");

    let resolved: Arc<LtxResolution> = self.project.ltx_project.system_ltx()?;
    let system_ltx: &Ltx = &resolved.ltx;
    let system_ltx_path: PathBuf = self.project.ltx_project.system_ltx_report_path()?;
    let player_hud_sections: Vec<(&str, &Section)> = system_ltx
      .iter()
      .filter(|(_, section)| GamedataItemSection::is_player_hud(section))
      .collect();

    let read: Vec<(String, PlayerHudMotions, Vec<String>)> = player_hud_sections
      .par_iter()
      .filter(|_| !self.options.job.is_cancelled())
      .map(|(section_name, section)| {
        let mut problems: Vec<String> = Vec::new();
        let motions: PlayerHudMotions = self.read_motions(section_name, section, &mut problems);

        (section_name.to_string(), motions, problems)
      })
      .collect();

    self.options.job.check_cancelled()?;

    let mut invalid_hands: BTreeSet<String> = BTreeSet::new();
    let mut messages: Vec<String> = Vec::new();
    let mut hands: Vec<(String, PlayerHudMotions)> = Vec::with_capacity(read.len());

    for (section_name, motions, problems) in read {
      if !problems.is_empty() {
        invalid_hands.insert(section_name.clone());
        messages.extend(problems);
      }

      // Hands playing nothing were reported above; judging every weapon's motions against them would only repeat it.
      if !motions.is_empty() {
        hands.push((section_name, motions));
      }
    }

    let verdict: PlayerHudMotionVerdict =
      PlayerHudMotionRequests::collect(system_ltx).verify(&hands, self.project.engine.engine);

    invalid_hands.extend(verdict.invalid_hands);
    messages.extend(verdict.messages);

    let mut findings: Vec<Finding> = messages
      .into_iter()
      .map(|message| {
        GamedataFindingFactory::for_asset(GamedataVerificationRule::AnimationsPlayerHud, &system_ltx_path, message)
      })
      .collect();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    for finding in &findings {
      xrf_output::error!(self.options.output, "{}", finding.message());
    }

    let checked_huds_count: u32 = Self::narrow_count(player_hud_sections.len())?;
    let invalid_huds_count: u32 = Self::narrow_count(invalid_hands.len())?;

    xrf_output::info!(
      self.options.output,
      "Verified gamedata player HUD sections, {}/{} valid",
      checked_huds_count - invalid_huds_count,
      checked_huds_count,
    );

    Ok(GamedataPlayerHudAnimationsVerificationResult {
      checked_huds_count,
      findings,
      invalid_huds_count,
    })
  }

  fn narrow_count(count: usize) -> XrfResult<u32> {
    u32::try_from(count).map_err(|_| XrfError::new_verify_error("Player HUD count exceeds the supported result range"))
  }

  /// The motions the hands visual plays, each problem reading them added to `problems`. Resolved and read through the
  /// VFS, so an archived hands visual and its banks are checked too.
  fn read_motions(&self, section_name: &str, section: &Section, problems: &mut Vec<String>) -> PlayerHudMotions {
    let mut motions: BTreeSet<String> = BTreeSet::new();
    let visual: &str = section.get("visual").unwrap_or_default();

    let Some(visual_path) = self
      .project
      .ogf(visual)
      .ok()
      .flatten()
      .map(|location| location.get_logical_path().to_string())
    else {
      problems.push(format!(
        "Player HUD section [{section_name}] visual '{visual}' is not found"
      ));

      return PlayerHudMotions::default();
    };

    let refs: Vec<String> = match GamedataVisualMotions::read_refs(self.project, &visual_path) {
      Ok(Some(refs)) => refs,
      Ok(None) => Vec::new(),
      Err(error) => {
        problems.push(format!(
          "Player HUD section [{section_name}] visual '{visual_path}' motion refs do not read: {error}"
        ));

        return PlayerHudMotions::default();
      }
    };

    // A ref that resolves to no bank is the meshes check's finding, against the visual that names it.
    for motion_ref in &refs {
      for bank in GamedataVisualMotions::resolve_banks(self.project, motion_ref).unwrap_or_default() {
        match GamedataVisualMotions::read_names(self.project, &bank) {
          Ok(names) if names.is_empty() => problems.push(format!(
            "Player HUD section [{section_name}] motion bank '{bank}' holds no motions"
          )),
          Ok(names) => motions.extend(names),
          Err(error) => problems.push(format!(
            "Player HUD section [{section_name}] motion bank '{bank}' does not read: {error}"
          )),
        }
      }
    }

    // One message for hands that play nothing, unless a problem above already says why.
    if motions.is_empty() && problems.is_empty() {
      problems.push(format!("Player HUD section [{section_name}] has no animations"));
    }

    PlayerHudMotions::new(motions)
  }
}
