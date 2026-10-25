use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxResolution, Section};

use crate::project::animations::hud::hud_motion_collisions_verification_result::GamedataHudMotionCollisionsVerificationResult;
use crate::project::gamedata_item_section::GamedataItemSection;
use crate::project::gamedata_visual_motions::GamedataVisualMotions;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Verifies that motion names stay unique within the set of banks a hands model loads.
pub(crate) struct HudMotionCollisionsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> HudMotionCollisionsVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataHudMotionCollisionsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::verbose!(self.options.output, "Verify hud motion collisions");

    let resolved: Arc<LtxResolution> = self.project.ltx_project.system_ltx()?;
    let system_ltx: &Ltx = &resolved.ltx;
    let system_ltx_path: PathBuf = self.project.ltx_project.system_ltx_report_path()?;

    let hud_sections: Vec<(&str, &Section)> = system_ltx
      .iter()
      .filter(|(_, section)| GamedataItemSection::is_player_hud(section))
      .collect();

    let checked_huds_count: u32 = u32::try_from(hud_sections.len())
      .map_err(|_| XrfError::new_verify_error("Player HUD count exceeds the supported result range"))?;

    let mut messages: Vec<String> = hud_sections
      .par_iter()
      .filter(|_| !self.options.job.is_cancelled())
      .flat_map(|(_, section)| self.collect_collisions(section))
      .collect();

    // Hands models usually share one bank directory, so the same collision is reported by each of
    // them. It is a property of the banks, not of any single hands model, so report it once.
    messages.sort();
    messages.dedup();

    let collisions_count: u32 = u32::try_from(messages.len())
      .map_err(|_| XrfError::new_verify_error("Motion collision count exceeds the supported result range"))?;

    xrf_output::info!(
      self.options.output,
      "Verified gamedata hud motion namespaces, {} collisions across {} huds",
      collisions_count,
      checked_huds_count,
    );

    let mut findings: Vec<Finding> = messages
      .into_iter()
      .map(|message| {
        GamedataFindingFactory::for_asset(
          GamedataVerificationRule::AnimationsMotionCollision,
          &system_ltx_path,
          message,
        )
      })
      .collect();

    self.options.job.check_cancelled()?;

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    self.options.job.check_cancelled()?;

    Ok(GamedataHudMotionCollisionsVerificationResult {
      checked_huds_count,
      collisions_count,
      findings,
    })
  }

  /// Report each motion name provided by more than one of the banks this hud section loads.
  fn collect_collisions(&self, section: &Section) -> Vec<String> {
    let Some(visual) = section.get("visual") else {
      return Vec::new();
    };

    // Resolved and read through the VFS, so an archived visual and its animation banks are checked too.
    let Ok(Some(visual_path)) = self
      .project
      .ogf(visual)
      .map(|it| it.map(|location| location.get_logical_path().to_string()))
    else {
      return Vec::new();
    };

    // A visual or bank that does not read is the player HUD check's finding, not a collision.
    let Ok(Some(refs)) = GamedataVisualMotions::read_refs(self.project, &visual_path) else {
      return Vec::new();
    };

    let banks: BTreeSet<String> = refs
      .iter()
      .filter_map(|motion_ref| GamedataVisualMotions::resolve_banks(self.project, motion_ref).ok())
      .flatten()
      .collect();

    // Ordered so the reported bank list is stable between runs.
    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for bank in &banks {
      if self.options.job.is_cancelled() {
        break;
      }

      let Ok(motions) = GamedataVisualMotions::read_names(self.project, bank) else {
        continue;
      };

      // The bank's file name, taken from the logical path: reports name the omf, not its whole path.
      let bank_name: String = bank.rsplit('\\').next().unwrap_or(bank).to_string();

      for motion in motions {
        if self.options.job.is_cancelled() {
          break;
        }

        owners.entry(motion).or_default().push(bank_name.clone());
      }
    }

    owners
      .into_iter()
      .filter(|(_, banks)| banks.len() > 1)
      .map(|(motion, mut banks)| {
        banks.sort();

        format!(
          "Motion '{motion}' is defined by {} linked banks, only one of them will be used: {}",
          banks.len(),
          banks.join(", ")
        )
      })
      .collect()
  }
}
