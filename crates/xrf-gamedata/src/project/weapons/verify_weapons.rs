use std::time::{Duration, Instant};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use xrf_db::{OgfFile, OmfFile, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_ltx::{LTX_SYMBOL_SCHEME, Ltx, Section};
use xrf_output::{OutputSequence, OutputSlot};
use xrf_vfs::XrayAssetType;
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::weapons::constants::NO_SOUND;
use crate::project::weapons::verify_weapons_result::GamedataWeaponVerificationResult;
use crate::project::weapons::weapon_sound_layer_issues::{WeaponSoundLayerIssue, weapon_sound_layer_issues};
use crate::project::weapons::weapon_sound_source::WeaponSoundSource;
use crate::project::weapons::weapon_sound_value::WeaponSoundValue;
use crate::project::weapons::weapons_utils::{get_weapon_animation_name, is_weapon_section};
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  pub fn verify_weapons(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<GamedataWeaponVerificationResult> {
    xrf_output::heading!(options.output, "Verify weapons:");

    let started_at: Instant = Instant::now();
    let system_ltx: Ltx = self.ltx_project.system_ltx()?;
    let system_ltx_path = self.ltx_project.system_ltx_report_path()?;

    // Selected before the sweep rather than filtered inside it, because a parallel run needs to know how many positions
    // it is releasing output through before the first of them says anything.
    let weapons: Vec<(&str, &Section)> = system_ltx
      .iter()
      .filter(|(_, section)| is_weapon_section(section))
      .collect();

    let checked_weapons_count: u32 = weapons.len() as u32;

    // Sections are independent once `system.ltx` is assembled: each reads its own visuals, sounds and animations and
    // writes nothing back. This was the longest strictly serial workload in a sweep, and once the checks themselves
    // began to overlap it was what the whole run waited on.
    let sequence: OutputSequence = OutputSequence::new(&options.output, weapons.len());

    let mut findings: Vec<Finding> = weapons
      .par_iter()
      .enumerate()
      .flat_map(|(index, (section_name, section))| {
        let slot: OutputSlot = sequence.new_slot(index);
        let scoped: GamedataProjectVerifyOptions = options.with_output(slot.get_output().clone());

        match self.verify_ltx_weapon(&scoped, &system_ltx, section_name, section) {
          Ok(true) => None,
          Ok(false) => {
            xrf_output::error!(scoped.output, "Invalid weapon section: [{section_name}]");

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::WeaponsValidation,
              &system_ltx_path,
              format!("Weapon section [{section_name}] is invalid"),
            ))
          }
          Err(error) => {
            xrf_output::error!(
              scoped.output,
              "Invalid weapon section: [{section_name}], failure: {error:?}"
            );

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::WeaponsValidation,
              &system_ltx_path,
              format!("Weapon section [{section_name}] failed verification: {error}"),
            ))
          }
        }
      })
      .collect();

    // One finding per invalid section, so the count is what the sweep collected rather than a second tally that could
    // disagree with it.
    let invalid_weapons_count: u32 = findings.len() as u32;

    let duration: Duration = started_at.elapsed();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    xrf_output::info!(
      options.output,
      "Verified gamedata weapons in {}, {}/{} valid",
      xrf_utils::format_duration(duration),
      checked_weapons_count - invalid_weapons_count,
      checked_weapons_count
    );

    Ok(GamedataWeaponVerificationResult {
      duration,
      checked_weapons_count,
      findings,
      invalid_weapons_count,
    })
  }

  pub fn verify_ltx_weapon(
    &self,
    options: &GamedataProjectVerifyOptions,
    ltx: &Ltx,
    section_name: &str,
    section: &Section,
  ) -> XrfResult<bool> {
    xrf_output::verbose!(options.output, "Verify weapon ltx config [{section_name}]");

    let mut is_weapon_valid: bool = true;

    // todo: Check animations as separate util checker for all existing meshes.
    // todo: Check textures as separate util checker for all existing meshes.

    if !self
      .verify_weapon_hud(options, ltx, section_name, section)
      .is_ok_and(|it| it)
    {
      is_weapon_valid = false;
    }

    if !self
      .verify_weapon_sounds(options, ltx, section_name, section)
      .is_ok_and(|it| it)
    {
      is_weapon_valid = false;
    }

    if !self
      .verify_weapon_bore_animations(options, ltx, section_name, section)
      .is_ok_and(|it| it)
    {
      is_weapon_valid = false;
    }

    Ok(is_weapon_valid)
  }

  /// Verify that a launcher capable weapon keeps its grenade launcher bore animations defined.
  ///
  /// `CWeaponMagazinedWGrenade::PlayAnimBore` switches to `anm_bore_g` or `anm_bore_w_gl` as soon as
  /// a launcher is attached. The Anomaly engine detects a missing bore and returns to idle, but
  /// OpenXRay only falls back to `anim_` prefixed aliases that this project never defines. With
  /// nothing to play the animation end callback never fires and the weapon stays in the bore state,
  /// so these keys cannot be dropped the way Anomaly content drops them.
  pub fn verify_weapon_bore_animations(
    &self,
    options: &GamedataProjectVerifyOptions,
    ltx: &Ltx,
    section_name: &str,
    section: &Section,
  ) -> XrfResult<bool> {
    // Only weapons that can carry a launcher ever reach the grenade launcher bore branch.
    if section.get("grenade_launcher_status").is_none_or(|it| it.trim() == "0") {
      return Ok(true);
    }

    let Some(hud_section) = section.get("hud").and_then(|it| ltx.section(it)) else {
      return Ok(true);
    };

    // A weapon without any bore never enters the state, so there is nothing to keep consistent.
    if hud_section.get("anm_bore").is_none() {
      return Ok(true);
    }

    let mut is_valid: bool = true;

    for required in ["anm_bore_g", "anm_bore_w_gl"] {
      if hud_section.get(required).is_none() {
        xrf_output::error!(
          options.output,
          "Weapon [{section_name}] supports a grenade launcher and defines anm_bore, but its hud section has no {required}, and the engine has no fallback for it"
        );

        is_valid = false;
      }
    }

    Ok(is_valid)
  }

  pub fn verify_weapon_hud(
    &self,
    options: &GamedataProjectVerifyOptions,
    ltx: &Ltx,
    section_name: &str,
    section: &Section,
  ) -> XrfResult<bool> {
    let mut is_valid: bool = true;

    // Resolved and read through the VFS, so a visual inside an archive volume is checked too.
    if let Some(visual) = &section.get("visual").and_then(|it| {
      self
        .ogf(it)
        .ok()
        .flatten()
        .map(|location| location.get_logical_path().to_string())
    }) {
      if let Err(error) = self.read_parsed(AssetType::Ogf, visual, |chunk| {
        OgfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
      }) {
        xrf_output::error!(
          options.output,
          "Failed to read weapon visual: [{}] - {:?} - {}",
          section_name,
          section.get("visual"),
          error
        );

        is_valid = false;
      }
    } else {
      xrf_output::error!(
        options.output,
        "Not found weapon visual: [{}] - {:?}",
        section_name,
        section.get("visual")
      );

      is_valid = false;
    }

    let hud_section: &Section = match section.get("hud").and_then(|it| ltx.section(it)) {
      Some(it) => it,
      None => {
        xrf_output::error!(
          options.output,
          "Not found hud section: [{}] - {:?}",
          section_name,
          section.get("hud")
        );

        return Ok(false);
      }
    };

    if let Some(visual_path) = &hud_section.get("item_visual").and_then(|it| {
      self
        .ogf(it)
        .ok()
        .flatten()
        .map(|location| location.get_logical_path().to_string())
    }) {
      match self.read_parsed(AssetType::Ogf, visual_path, |chunk| {
        OgfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
      }) {
        Ok(hud_visual) => {
          if let Some(motion_refs) = hud_visual.kinematics.as_ref().map(|it| &it.motion_refs) {
            let mut ref_animations: Vec<String> = Vec::new();

            for motion_ref in motion_refs {
              if let Some(motion_file_path) = self
                .omf(motion_ref)
                .ok()
                .flatten()
                .map(|location| location.get_logical_path().to_string())
              {
                // The whole bank, which the meshes and animations checks read too: shared banks are the reason this
                // check spent most of its time re-reading the same megabytes.
                match self
                  .read_parsed(AssetType::Omf, &motion_file_path, |chunk| {
                    OmfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
                  })
                  .map(|omf| {
                    omf
                      .get_motion_names()
                      .into_iter()
                      .map(str::to_owned)
                      .collect::<Vec<String>>()
                  }) {
                  Ok(motions) => ref_animations.extend(motions),
                  Err(error) => {
                    xrf_output::error!(
                      options.output,
                      "Error reading OMF motions for weapon hud: [{}] : {} - {}",
                      section_name,
                      visual_path,
                      error
                    );

                    is_valid = false;
                  }
                }
              } else {
                xrf_output::error!(
                  options.output,
                  "Error reading OMF motions for weapon hud: [{}] : {}, no asset found",
                  section_name,
                  visual_path
                );

                is_valid = false;
              }
            }

            for (field_name, field_value) in hud_section {
              if !field_name.starts_with("anm_") {
                continue;
              }

              let animation_name: String = get_weapon_animation_name(field_value);

              if !ref_animations.contains(&animation_name) {
                // todo: Check available motions from outfit sections here.
              }
            }
          } else {
            xrf_output::error!(
              options.output,
              "Missing motion refs for weapon hud: [{}] : {}",
              section_name,
              visual_path
            );

            is_valid = false;
          }
        }
        Err(error) => {
          xrf_output::error!(
            options.output,
            "Failed to read weapon hud visual: [{}] - {:?} - {}",
            section_name,
            section.get("visual"),
            error
          );

          is_valid = false;
        }
      }
    } else {
      xrf_output::error!(options.output, "Not found hud visual definition: [{section_name}]");

      is_valid = false;
    }

    Ok(is_valid)
  }

  pub fn verify_weapon_sounds(
    &self,
    options: &GamedataProjectVerifyOptions,
    ltx: &Ltx,
    section_name: &str,
    section: &Section,
  ) -> XrfResult<bool> {
    let mut are_sounds_valid: bool = true;

    for sound_section in ["snd_draw", "snd_empty", "snd_holster", "snd_reload", "snd_shoot"] {
      if !section.contains_key(sound_section) {
        xrf_output::error!(
          options.output,
          "Missing section required weapon sound: [{section_name}] : {sound_section}"
        );

        are_sounds_valid = false;
      }
    }

    for (field_name, field_value) in section {
      if !field_name.starts_with("snd_") {
        continue;
      }

      let value: WeaponSoundValue<'_> = WeaponSoundValue::parse(field_value);
      let source: WeaponSoundSource<'_, '_> = WeaponSoundSource::classify(ltx, value);

      match source {
        WeaponSoundSource::Asset { name } if name == NO_SOUND => continue,
        WeaponSoundSource::Asset { name } => {
          if !self
            .verify_weapon_sound_asset(options, section_name, field_name, name)
            .is_ok_and(|it| it)
          {
            are_sounds_valid = false;
          }
        }
        WeaponSoundSource::LayeredSection {
          name,
          has_parameters,
          section,
        } => {
          if has_parameters {
            xrf_output::error!(
              options.output,
              "Layered sound reference cannot include volume or delay: [{section_name}] {field_name} : {field_value}"
            );

            are_sounds_valid = false;
          }

          if !self
            .verify_weapon_sound_layer(options, name, section)
            .is_ok_and(|it| it)
          {
            are_sounds_valid = false;
          }
        }
      }
    }

    Ok(are_sounds_valid)
  }

  pub fn verify_weapon_sound_layer(
    &self,
    options: &GamedataProjectVerifyOptions,
    section_name: &str,
    section: &Section,
  ) -> XrfResult<bool> {
    let issues: Vec<WeaponSoundLayerIssue> = weapon_sound_layer_issues(section);
    let mut is_valid: bool = issues.is_empty();

    for issue in issues {
      match issue {
        WeaponSoundLayerIssue::InvalidFieldName { field_name } => {
          let field_value: Option<&str> = section.get(&field_name);

          xrf_output::error!(
            options.output,
            "Sound layer field name is invalid, should match pattern: [{section_name}] {field_name} : {field_value:?}"
          );
        }
        WeaponSoundLayerIssue::MissingLayer {
          expected,
          found: Some(found),
        } => {
          xrf_output::error!(
            options.output,
            "Sound layer section has a gap: [{section_name}] expected snd_{expected}_layer before snd_{found}_layer"
          );
        }
        WeaponSoundLayerIssue::MissingLayer { expected, found: None } => {
          xrf_output::error!(
            options.output,
            "Sound layer section is missing required first layer: [{section_name}] snd_{expected}_layer"
          );
        }
        WeaponSoundLayerIssue::MissingBaseLayer { layer } => {
          xrf_output::error!(
            options.output,
            "Sound layer variants require a base layer: [{section_name}] snd_{layer}_layer"
          );
        }
        WeaponSoundLayerIssue::MissingVariant { layer, expected, found } => {
          xrf_output::error!(
            options.output,
            "Sound layer variants have a gap: [{section_name}] expected snd_{layer}_layer{expected} before snd_{layer}_layer{found}"
          );
        }
      }
    }

    for (field_name, field_value) in section {
      // Metadata fields such as `$scheme` describe the section, they are not sound references.
      if field_name.starts_with(LTX_SYMBOL_SCHEME) {
        continue;
      }

      if !self
        .verify_weapon_sound_asset(options, section_name, field_name, field_value)
        .is_ok_and(|it| it)
      {
        is_valid = false
      }
    }

    if is_valid {
      xrf_output::verbose!(options.output, "Sound layers section verified: [{section_name}]");
    }

    Ok(is_valid)
  }

  fn verify_weapon_sound_asset(
    &self,
    options: &GamedataProjectVerifyOptions,
    section_name: &str,
    field_name: &str,
    field_value: &str,
  ) -> XrfResult<bool> {
    let mut is_valid: bool = true;

    // Sounds field is 1-3 comma separated values. The reference may name the sound with or without its extension, which
    let sound_object_value: String = get_weapon_animation_name(field_value);

    // Resolving through the VFS *is* the existence check, loose or archived, so there is no second filesystem probe to make.
    //
    // todo: Check the OGG contents, not only that the sound resolves.
    if self
      .resolve(XrayAssetType::Ogg, &sound_object_value)
      .ok()
      .flatten()
      .is_some()
    {
      xrf_output::verbose!(
        options.output,
        "Sound verified in section: [{section_name}] : {field_name} -> {sound_object_value}"
      );
    } else {
      xrf_output::error!(
        options.output,
        "Sound not found in section: [{section_name}] : {field_name} -> {sound_object_value}"
      );

      is_valid = false;
    }

    Ok(is_valid)
  }
}
