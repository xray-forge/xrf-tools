use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxResolution, Section};

use crate::project::animations::hud::hud_item_animations_verification_result::GamedataHudItemAnimationsVerificationResult;
use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::gamedata_item_section::GamedataItemSection;
use crate::project::gamedata_visual_motions::GamedataVisualMotions;
use crate::{GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Motion the engine plays on the item model, and the `anm_*` field that asked for it.
struct ExpectedItemMotion {
  field_name: String,
  motion_name: String,
}

/// Verifies motions that an item model is explicitly asked to play.
pub(crate) struct HudItemAnimationsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> HudItemAnimationsVerifier<'a> {
  /// Motion every item model must provide, used by the engine as a fallback.
  const FALLBACK_MOTION: &'static str = "idle";

  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataHudItemAnimationsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::verbose!(self.options.output, "Verify hud item animations");

    let resolved: Arc<LtxResolution> = self.project.ltx_project.system_ltx()?;
    let system_ltx: &Ltx = &resolved.ltx;
    let system_ltx_path: PathBuf = self.project.ltx_project.system_ltx_report_path()?;

    let item_sections: Vec<(&str, &Section)> = system_ltx
      .iter()
      .filter(|(_, section)| GamedataItemSection::is_hud_item(section))
      .collect();

    // Each section is verified once, reading its model and omf files a single time.
    let outcome: GamedataAssetSweepOutcome =
      GamedataAssetSweep::new(self.options).run(&item_sections, |_, (section_name, section)| {
        self
          .verify_item_animations(section_name, section)
          .into_iter()
          .map(|message| {
            GamedataFindingFactory::for_asset(GamedataVerificationRule::AnimationsHudItem, &system_ltx_path, message)
          })
          .collect()
      })?;

    self.options.job.check_cancelled()?;

    xrf_output::info!(
      self.options.output,
      "Verified gamedata hud items, {}/{} valid",
      outcome.checked_count - outcome.invalid_count,
      outcome.checked_count,
    );

    Ok(GamedataHudItemAnimationsVerificationResult {
      checked_items_count: outcome.checked_count,
      findings: outcome.findings,
      invalid_items_count: outcome.invalid_count,
    })
  }

  /// Verify single hud item section, returning a message per problem found.
  fn verify_item_animations(&self, section_name: &str, section: &Section) -> Vec<String> {
    let Some(item_visual) = section.get("item_visual") else {
      return Vec::new();
    };

    let item_motions: BTreeSet<String> = match self.read_model_motions(item_visual) {
      Ok(Some(motions)) => motions,
      // Static model, the engine plays no item animations for it at all.
      Ok(None) => return Vec::new(),
      Err(error) => {
        return vec![format!(
          "Hud item section [{section_name}] visual '{item_visual}' motions were not read: {error}"
        )];
      }
    };

    let mut messages: Vec<String> = Vec::new();

    if !item_motions.contains(Self::FALLBACK_MOTION) {
      messages.push(format!(
        "Hud item section [{section_name}] visual '{item_visual}' has no '{}' motion, the engine asserts on it as animation fallback",
        Self::FALLBACK_MOTION
      ));
    }

    for expected in Self::collect_expected_item_motions(section) {
      if self.options.job.is_cancelled() {
        break;
      }

      if !item_motions.contains(&expected.motion_name) {
        messages.push(format!(
          "Hud item section [{section_name}] {}={} -> explicitly requested item motion is not found in '{item_visual}'",
          expected.field_name, expected.motion_name
        ));
      }
    }

    messages
  }

  /// Collect item motions the section explicitly asks for through a second animation token.
  fn collect_expected_item_motions(section: &Section) -> Vec<ExpectedItemMotion> {
    let mut expected: Vec<ExpectedItemMotion> = Vec::new();

    for (field_name, field_value) in section {
      if !field_name.starts_with("anm_") && !field_name.starts_with("anim_") {
        continue;
      }

      let items: Vec<&str> = field_value.split(',').map(|it| it.trim()).collect();
      let Some(base_name) = items.first().copied() else {
        continue;
      };

      // Only an explicit second token names an item motion. Without one the engine reuses the hands
      // motion name and falls back to idle when the item has nothing to play, which is expected.
      if let Some(additional_name) = items.get(1).copied()
        && !additional_name.is_empty()
        && additional_name != base_name
      {
        expected.push(ExpectedItemMotion {
          field_name: field_name.into(),
          motion_name: additional_name.into(),
        });
      }
    }

    expected
  }

  /// Read motions provided by the model, or `None` when the model carries no animations at all.
  fn read_model_motions(&self, visual: &str) -> XrfResult<Option<BTreeSet<String>>> {
    self.options.job.check_cancelled()?;

    // Resolved and read through the VFS, so a visual and its animations inside archive volumes are checked too.
    let Some(visual_path) = self
      .project
      .ogf(visual)?
      .map(|location| location.get_logical_path().to_string())
    else {
      return Err(XrfError::new_not_found_error(format!(
        "Visual '{visual}' was not found"
      )));
    };

    GamedataVisualMotions::read_all(self.project, &visual_path, &self.options.job)
  }
}
