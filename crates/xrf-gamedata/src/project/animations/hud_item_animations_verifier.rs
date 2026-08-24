use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use rayon::prelude::*;
use xrf_chunk::ChunkReader;
use xrf_db::{OgfFile, OmfFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::animations::hud_item_animations_verification_result::GamedataHudItemAnimationsVerificationResult;
use crate::project::weapons::weapons_utils::is_hud_item_section;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

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
    xrf_output::verbose!(self.options.output, "Verify hud item animations");

    let system_ltx: Ltx = self.project.ltx_project.system_ltx()?;
    let system_ltx_path: PathBuf = self.project.ltx_project.system_ltx_report_path()?;

    let item_sections: Vec<(&String, &Section)> = system_ltx
      .sections
      .iter()
      .filter(|(_, section)| is_hud_item_section(section))
      .collect();

    let checked_items_count: u32 = u32::try_from(item_sections.len())
      .map_err(|_| XrfError::new_verify_error("Hud item count exceeds the supported result range"))?;

    // Each section is verified once, reading its model and omf files a single time.
    let messages_per_item: Vec<Vec<String>> = item_sections
      .par_iter()
      .map(|(section_name, section)| self.verify_item_animations(section_name, section))
      .collect();

    let invalid_items_count: u32 = u32::try_from(messages_per_item.iter().filter(|it| !it.is_empty()).count())
      .map_err(|_| XrfError::new_verify_error("Invalid hud item count exceeds the supported result range"))?;

    let mut findings: Vec<Finding> = messages_per_item
      .into_iter()
      .flatten()
      .map(|message| {
        GamedataFindingFactory::for_asset(GamedataVerificationRule::AnimationsHudItem, &system_ltx_path, message)
      })
      .collect();

    xrf_output::info!(
      self.options.output,
      "Verified gamedata hud items, {}/{} valid",
      checked_items_count - invalid_items_count,
      checked_items_count,
    );

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    Ok(GamedataHudItemAnimationsVerificationResult {
      checked_items_count,
      findings,
      invalid_items_count,
    })
  }

  /// Verify single hud item section, returning a message per problem found.
  fn verify_item_animations(&self, section_name: &str, section: &Section) -> Vec<String> {
    let Some(item_visual) = section.get("item_visual") else {
      return Vec::new();
    };

    let item_motions: HashSet<String> = match self.read_model_motions(item_visual) {
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
  fn read_model_motions(&self, visual: &str) -> XrfResult<Option<HashSet<String>>> {
    // Resolved and read through the VFS, so a visual and its animations inside archive volumes are checked too.
    let Some(visual_path) = self
      .project
      .vfs()
      .scoped(&self.project.scope)
      .ogf(visual)?
      .map(|location| location.get_logical_path().to_string())
    else {
      return Err(XrfError::new_not_found_error(format!(
        "Visual '{visual}' was not found"
      )));
    };

    let Some(linked_assets) = self.read_motion_refs(&visual_path)? else {
      return Ok(None);
    };

    let mut motions: HashSet<String> = HashSet::new();

    for linked in linked_assets {
      let omf: Arc<OmfFile> = self.project.read_parsed(AssetType::Omf, &linked, |chunk| {
        OmfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
      })?;

      motions.extend(omf.get_motion_names().into_iter().map(str::to_owned));
    }

    Ok(Some(motions))
  }

  /// Resolve omf assets linked by the model motion refs, or `None` when the model has no refs.
  fn read_motion_refs(&self, path: &str) -> XrfResult<Option<HashSet<String>>> {
    // todo: Review why full read fails and use plain read_parsed.
    // Narrow read, assembled here rather than taken through the parsed seam: a full visual parse fails on visuals whose
    // geometry will not read, while their motion refs chunk reads fine, and this check must still see those refs.
    // `patch-ogf-motion-refs` depends on the same tolerance.
    let motion_refs: Vec<String> = match self
      .project
      .read_bytes(path)
      .and_then(ChunkReader::from_vec)
      .and_then(|mut chunk| OgfFile::read_motion_refs_from_chunk::<XRayByteOrder, _>(&mut chunk))
    {
      Ok(refs) => refs,
      // Model has no motion refs chunk, so it is a static visual without animations.
      Err(XrfError::NotFound { .. }) => return Ok(None),
      Err(error) => return Err(error),
    };

    let mut assets: HashSet<String> = HashSet::new();

    for motion_ref in &motion_refs {
      for location in self
        .project
        .vfs()
        .scoped(&self.project.scope)
        .resolve_all(AssetType::Omf, motion_ref)?
      {
        if location.is_type(AssetType::Omf) {
          assets.insert(location.get_logical_path().to_string());
        }
      }
    }

    Ok(Some(assets))
  }
}
