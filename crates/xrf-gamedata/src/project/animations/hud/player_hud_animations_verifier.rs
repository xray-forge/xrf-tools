use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxResolution, Section};

use crate::project::animations::hud::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::gamedata_item_section::GamedataItemSection;
use crate::project::gamedata_visual_motions::GamedataVisualMotions;
use crate::{GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

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

    let outcome: GamedataAssetSweepOutcome =
      GamedataAssetSweep::new(self.options).run(&player_hud_sections, |options, (section_name, section)| {
        xrf_output::verbose!(options.output, "Verify player hud config [{section_name}]");

        self
          .verify_player_hud(system_ltx, section_name, section)
          .into_iter()
          .map(|message| {
            xrf_output::error!(options.output, "{message}");

            GamedataFindingFactory::for_asset(GamedataVerificationRule::AnimationsPlayerHud, &system_ltx_path, message)
          })
          .collect()
      })?;

    self.options.job.check_cancelled()?;

    xrf_output::info!(
      self.options.output,
      "Verified gamedata huds, {}/{} valid",
      outcome.checked_count - outcome.invalid_count,
      outcome.checked_count,
    );

    Ok(GamedataPlayerHudAnimationsVerificationResult {
      checked_huds_count: outcome.checked_count,
      findings: outcome.findings,
      invalid_huds_count: outcome.invalid_count,
    })
  }

  /// Every problem with one player HUD section, a message each.
  fn verify_player_hud(&self, system_ltx: &Ltx, section_name: &str, section: &Section) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let motions: BTreeSet<String> = self.read_motions(section_name, section, &mut messages);

    if motions.is_empty() {
      messages.push(format!("Player HUD section [{section_name}] has no animations"));

      return messages;
    }

    for (weapon_name, weapon) in system_ltx.iter() {
      if self.options.job.is_cancelled() {
        break;
      }

      // A weapon without a HUD section the hands can play is reported by the weapons check.
      let Some(hud) = weapon
        .get("hud")
        .filter(|_| GamedataItemSection::is_weapon(weapon))
        .and_then(|hud| system_ltx.section(hud))
      else {
        continue;
      };

      for (field_name, field_value) in hud {
        if !field_name.starts_with("anm_") {
          continue;
        }

        let motion: &str = GamedataItemSection::get_motion_name(field_value);

        if !motions.contains(motion) {
          messages.push(format!(
            "Player HUD section [{section_name}] has no motion '{motion}', which weapon [{weapon_name}] plays as {field_name}"
          ));
        }
      }
    }

    messages
  }

  /// The motions the hands visual plays, each problem reading them added to `messages`. Resolved and read through the
  /// VFS, so an archived hands visual and its banks are checked too.
  fn read_motions(&self, section_name: &str, section: &Section, messages: &mut Vec<String>) -> BTreeSet<String> {
    let mut motions: BTreeSet<String> = BTreeSet::new();
    let visual: &str = section.get("visual").unwrap_or_default();

    let Some(visual_path) = self
      .project
      .ogf(visual)
      .ok()
      .flatten()
      .map(|location| location.get_logical_path().to_string())
    else {
      messages.push(format!(
        "Player HUD section [{section_name}] visual '{visual}' is not found"
      ));

      return motions;
    };

    let refs: Vec<String> = match GamedataVisualMotions::read_refs(self.project, &visual_path) {
      Ok(Some(refs)) => refs,
      Ok(None) => return motions,
      Err(error) => {
        messages.push(format!(
          "Player HUD section [{section_name}] visual '{visual_path}' motion refs do not read: {error}"
        ));

        return motions;
      }
    };

    // A ref that resolves to no bank is the meshes check's finding, against the visual that names it.
    for motion_ref in &refs {
      for bank in GamedataVisualMotions::resolve_banks(self.project, motion_ref).unwrap_or_default() {
        match GamedataVisualMotions::read_names(self.project, &bank) {
          Ok(names) if names.is_empty() => messages.push(format!(
            "Player HUD section [{section_name}] motion bank '{bank}' holds no motions"
          )),
          Ok(names) => motions.extend(names),
          Err(error) => messages.push(format!(
            "Player HUD section [{section_name}] motion bank '{bank}' does not read: {error}"
          )),
        }
      }
    }

    motions
  }
}
