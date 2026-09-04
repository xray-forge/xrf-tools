use std::collections::HashSet;
use std::path::PathBuf;

use rayon::prelude::*;
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_db::{OgfFile, OmfFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::animations::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
use crate::project::weapons::weapons_utils::{get_weapon_animation_name, is_player_hud_section, is_weapon_section};
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

pub(crate) struct PlayerHudAnimationsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> PlayerHudAnimationsVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataPlayerHudAnimationsVerificationResult> {
    xrf_output::verbose!(self.options.output, "Verify player hud animations");

    let system_ltx: Ltx = self.project.ltx_project.system_ltx()?;
    let system_ltx_path: PathBuf = self.project.ltx_project.system_ltx_report_path()?;
    let player_hud_sections: Vec<(&str, &Section)> = system_ltx
      .iter()
      .filter(|(_, section)| is_player_hud_section(section))
      .collect();

    let checked_huds_count: u32 = u32::try_from(player_hud_sections.len())
      .map_err(|_| XrfError::new_verify_error("Player HUD count exceeds the supported result range"))?;

    // Sections are verified in parallel, so each one logs into its listed position and the sequence
    // releases them in section order rather than in the order the workers finished.
    let sequence: OutputSequence = OutputSequence::new(&self.options.output, player_hud_sections.len());

    let mut findings: Vec<Finding> = player_hud_sections
      .par_iter()
      .enumerate()
      .filter_map(|(index, (section_name, section))| {
        let slot: OutputSlot = sequence.new_slot(index);
        let output: &OutputOptions = slot.get_output();

        xrf_output::verbose!(output, "Verify player hud config [{section_name}]");

        if self
          .verify_player_hud_animation(output, &system_ltx, section_name, section)
          .is_ok_and(|it| it)
        {
          return None;
        }

        xrf_output::info!(output, "Player hud config [{section_name}] is invalid");

        Some(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::AnimationsPlayerHud,
          &system_ltx_path,
          format!("Player HUD section [{section_name}] has invalid animations"),
        ))
      })
      .collect();

    let invalid_huds_count: u32 = u32::try_from(findings.len())
      .map_err(|_| XrfError::new_verify_error("Invalid player HUD count exceeds the supported result range"))?;

    xrf_output::info!(
      self.options.output,
      "Verified gamedata huds, {}/{} valid",
      checked_huds_count - invalid_huds_count,
      checked_huds_count,
    );

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    Ok(GamedataPlayerHudAnimationsVerificationResult {
      checked_huds_count,
      findings,
      invalid_huds_count,
    })
  }

  fn verify_player_hud_animation(
    &self,
    output: &OutputOptions,
    system_ltx: &Ltx,
    section_name: &str,
    section: &Section,
  ) -> XrfResult<bool> {
    let mut is_valid: bool = true;
    let mut hud_motions: HashSet<String> = HashSet::new();

    // Resolved and read through the VFS, so an archived hands visual and its banks are checked too.
    if let Some(visual_path) = &section.get("visual").and_then(|it| {
      self
        .project
        .vfs()
        .scoped(self.project.scope())
        .ogf(it)
        .ok()
        .flatten()
        .map(|location| location.get_logical_path().to_string())
    }) {
      xrf_output::verbose!(
        output,
        "Read player hud motion refs - [{}] {}",
        section_name,
        visual_path
      );

      match self.read_motion_refs(visual_path) {
        Ok(linked_visuals) => {
          xrf_output::verbose!(
            output,
            "Player hud ogf [{} contains {} linked omf files to check",
            visual_path,
            linked_visuals.len()
          );

          for linked_visual in &linked_visuals {
            match self
              .project
              .read_parsed(AssetType::Omf, linked_visual, |chunk| {
                OmfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
              })
              .map(|omf| {
                omf
                  .get_motion_names()
                  .into_iter()
                  .map(str::to_owned)
                  .collect::<Vec<String>>()
              }) {
              Ok(motions) => {
                if motions.is_empty() {
                  xrf_output::error!(output, "No motions in visual: [{}] - {}", section_name, linked_visual);

                  is_valid = false;
                }

                for motion in motions {
                  hud_motions.insert(motion);
                }
              }
              Err(error) => {
                xrf_output::error!(
                  output,
                  "Failed to read linked visual: [{}] - {} - {}",
                  section_name,
                  linked_visual,
                  error
                );

                is_valid = false;
              }
            }
          }
        }
        Err(error) => {
          xrf_output::error!(
            output,
            "Failed to read linked visuals: [{}] - {} - {}",
            section_name,
            visual_path,
            error
          );

          is_valid = false;
        }
      }
    } else {
      xrf_output::error!(
        output,
        "Not found hud visual: [{}] - {:?}",
        section_name,
        section.get("visual")
      );

      is_valid = false;
    }

    if hud_motions.is_empty() {
      xrf_output::error!(output, "Hud [{section_name}] contains no animations");

      is_valid = false;
    } else if !self
      .verify_weapon_animations(output, system_ltx, section_name, &hud_motions)
      .is_ok_and(|it| it)
    {
      xrf_output::error!(output, "Hud [{section_name}] failed weapons check");

      is_valid = false;
    }

    Ok(is_valid)
  }

  fn verify_weapon_animations(
    &self,
    output: &OutputOptions,
    system_ltx: &Ltx,
    section_name: &str,
    motions: &HashSet<String>,
  ) -> XrfResult<bool> {
    xrf_output::verbose!(output, "Verify weapons animations for [{section_name}]");

    let mut is_valid: bool = true;

    for (weapon_section_name, weapon_section) in system_ltx.iter() {
      if !is_weapon_section(weapon_section) {
        continue;
      }

      if let Some(hud_section_name) = weapon_section.get("hud") {
        if let Some(hud_section) = system_ltx.section(hud_section_name) {
          for (field_name, field_value) in hud_section {
            if !field_name.starts_with("anm_") {
              continue;
            }

            let weapon_motion_name: String = get_weapon_animation_name(field_value);

            if !motions.contains(&weapon_motion_name) {
              xrf_output::error!(
                output,
                "Hud [{section_name}] weapon [{weapon_section_name}] {field_name}={weapon_motion_name} -> animation motion is not found"
              );

              is_valid = false;
            }
          }
        } else {
          xrf_output::verbose!(
            output,
            "Not able to check weapon hud section [{section_name}] -> [{weapon_section_name}] [{hud_section_name}]"
          );
        }
      } else {
        xrf_output::verbose!(
          output,
          "Not able to check weapon hud [{section_name}] -> [{weapon_section_name}] hud"
        );
      }
    }

    Ok(is_valid)
  }

  fn read_motion_refs(&self, path: &str) -> XrfResult<HashSet<String>> {
    // todo: Fix and use read_parsed.
    // Narrow read: a full visual parse fails on visuals whose geometry will not read, while their motion refs chunk
    // reads fine, and this check must still see those refs.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(self.project.read_bytes(path)?)?;
    let motion_refs: Vec<String> = OgfFile::read_motion_refs_from_chunk::<XRayByteOrder, _>(&mut chunk)?;

    let mut assets: HashSet<String> = HashSet::new();

    for motion_ref in &motion_refs {
      for location in self
        .project
        .vfs()
        .scoped(self.project.scope())
        .resolve_all(AssetType::Omf, motion_ref)?
      {
        if location.is_type(AssetType::Omf) {
          assets.insert(location.get_logical_path().to_string());
        }
      }
    }

    Ok(assets)
  }
}
