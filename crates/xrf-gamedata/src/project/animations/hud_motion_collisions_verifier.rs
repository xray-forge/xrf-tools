use std::collections::BTreeMap;
use std::path::PathBuf;

use rayon::prelude::*;
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_db::{OgfFile, OmfFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::animations::hud_motion_collisions_verification_result::GamedataHudMotionCollisionsVerificationResult;
use crate::project::weapons::weapons_utils::is_player_hud_section;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Verifies that motion names stay unique within the set of banks a hands model loads.
///
/// The usual cause is a bank that was superseded but left in place next to its replacement.
pub struct HudMotionCollisionsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> HudMotionCollisionsVerifier<'a> {
  pub fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub fn verify(&self) -> XrfResult<GamedataHudMotionCollisionsVerificationResult> {
    xrf_output::verbose!(self.options.output, "Verify hud motion collisions");

    let system_ltx: Ltx = self.project.ltx_project.system_ltx()?;
    let system_ltx_path: PathBuf = self.project.ltx_project.system_ltx_report_path()?;

    let hud_sections: Vec<(&str, &Section)> = system_ltx
      .iter()
      .filter(|(_, section)| is_player_hud_section(section))
      .collect();

    let checked_huds_count: u32 = u32::try_from(hud_sections.len())
      .map_err(|_| XrfError::new_verify_error("Player HUD count exceeds the supported result range"))?;

    let mut messages: Vec<String> = hud_sections
      .par_iter()
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

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

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
      .vfs()
      .scoped(self.project.scope())
      .ogf(visual)
      .map(|it| it.map(|location| location.get_logical_path().to_string()))
    else {
      return Vec::new();
    };

    let Ok(banks) = self.read_motion_refs(&visual_path) else {
      return Vec::new();
    };

    // Ordered so the reported bank list is stable between runs.
    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for bank in &banks {
      // The whole bank, shared with the meshes and hud-item checks rather than parsed once per reader.
      let Ok(omf) = self.project.read_parsed(AssetType::Omf, bank, |chunk| {
        OmfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
      }) else {
        continue;
      };

      let motions: Vec<String> = omf.get_motion_names().into_iter().map(str::to_owned).collect();

      // The bank's file name, taken from the logical path: reports name the omf, not its whole path.
      let bank_name: String = bank.rsplit('\\').next().unwrap_or(bank).to_string();

      for motion in motions {
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

  /// Resolve omf assets linked by the model motion refs, wildcards included.
  fn read_motion_refs(&self, path: &str) -> XrfResult<Vec<String>> {
    let mut assets: Vec<String> = Vec::new();
    // todo: Review why full read fails and use plain read_parsed.
    // Narrow read: a full visual parse fails on visuals whose geometry will not read, while their motion refs chunk
    // reads fine, and this check must still see those refs.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(self.project.read_bytes(path)?)?;

    for motion_ref in &OgfFile::read_motion_refs_from_chunk::<XRayByteOrder, _>(&mut chunk)? {
      for location in self
        .project
        .vfs()
        .scoped(self.project.scope())
        .resolve_all(AssetType::Omf, motion_ref)?
      {
        if location.is_type(AssetType::Omf) {
          assets.push(location.get_logical_path().to_string());
        }
      }
    }

    assets.sort();
    assets.dedup();

    Ok(assets)
  }
}
