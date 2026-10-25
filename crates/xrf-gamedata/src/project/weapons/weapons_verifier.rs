use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_ltx::{LTX_SYMBOL_SCHEME, Ltx, LtxResolution, Section};
use xrf_ogf::OgfFile;
use xrf_spawn::XRayByteOrder;
use xrf_vfs::XrayAssetType;

use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_item_section::GamedataItemSection;
use crate::project::gamedata_visual_motions::GamedataVisualMotions;
use crate::project::weapons::verify_weapons_result::GamedataWeaponVerificationResult;
use crate::project::weapons::weapon_sound_layer_issue::WeaponSoundLayerIssue;
use crate::project::weapons::weapon_sound_source::WeaponSoundSource;
use crate::project::weapons::weapon_sound_value::WeaponSoundValue;
use crate::{GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// The weapons check: every weapon section's visuals, HUD, motion banks and sounds resolve as the engine loads them.
///
/// Each problem is a finding of its own, saying what is wrong with which section, and a section is invalid when it has
/// any.
pub(crate) struct WeaponsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for WeaponsVerifier<'a> {
  type Result = GamedataWeaponVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataWeaponVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify weapons:");

    let started_at: Instant = Instant::now();
    let resolved: Arc<LtxResolution> = self.project.ltx_project.system_ltx()?;
    let system_ltx: &Ltx = &resolved.ltx;
    let system_ltx_path: PathBuf = self.project.ltx_project.system_ltx_report_path()?;

    let weapons: Vec<(&str, &Section)> = system_ltx
      .iter()
      .filter(|(_, section)| GamedataItemSection::is_weapon(section))
      .collect();

    // Sections are independent once `system.ltx` is assembled: each reads its own visuals, sounds and animations and
    // writes nothing back.
    let outcome: GamedataAssetSweepOutcome =
      GamedataAssetSweep::new(self.options).run(&weapons, |options, (section_name, section)| {
        xrf_output::verbose!(options.output, "Verify weapon ltx config [{section_name}]");

        self
          .verify_weapon(system_ltx, section_name, section)
          .into_iter()
          .map(|message| {
            xrf_output::error!(options.output, "{message}");

            GamedataFindingFactory::for_asset(GamedataVerificationRule::WeaponsValidation, &system_ltx_path, message)
          })
          .collect()
      })?;

    self.options.job.check_cancelled()?;

    let duration: Duration = started_at.elapsed();

    xrf_output::info!(
      self.options.output,
      "Verified gamedata weapons in {}, {}/{} valid",
      xrf_utils::format_duration(duration),
      outcome.checked_count - outcome.invalid_count,
      outcome.checked_count
    );

    Ok(GamedataWeaponVerificationResult {
      duration,
      checked_weapons_count: outcome.checked_count,
      findings: outcome.findings,
      invalid_weapons_count: outcome.invalid_count,
    })
  }
}

impl WeaponsVerifier<'_> {
  /// Sounds the engine plays for every weapon, so a section without one is incomplete.
  const REQUIRED_SOUNDS: [&'static str; 5] = ["snd_draw", "snd_empty", "snd_holster", "snd_reload", "snd_shoot"];

  /// Grenade launcher bores a launcher-capable weapon defining `anm_bore` needs, which the engine has no fallback for.
  const LAUNCHER_BORES: [&'static str; 2] = ["anm_bore_g", "anm_bore_w_gl"];

  /// Every problem with one weapon section, a message each naming the section.
  fn verify_weapon(&self, ltx: &Ltx, name: &str, section: &Section) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();

    self.verify_visual(name, section, &mut problems);

    match section.get("hud").and_then(|hud| ltx.section(hud).map(|it| (hud, it))) {
      Some((hud_name, hud)) => {
        self.verify_hud(name, hud_name, hud, &mut problems);
        Self::verify_bores(name, section, hud, &mut problems);
      }
      None => problems.push(format!(
        "Weapon section [{name}] hud section '{}' is not defined",
        section.get("hud").unwrap_or_default()
      )),
    }

    self.verify_sounds(ltx, name, section, &mut problems);

    problems
  }

  /// The world visual resolves and reads, through the VFS so an archived visual is checked too.
  fn verify_visual(&self, name: &str, section: &Section, problems: &mut Vec<String>) {
    let visual: &str = section.get("visual").unwrap_or_default();

    match self.find(XrayAssetType::Ogf, visual) {
      Some(path) => {
        if let Err(error) = self
          .project
          .read_parsed(&path, OgfFile::read_from_chunk::<XRayByteOrder, _>)
        {
          problems.push(format!(
            "Weapon section [{name}] visual '{visual}' does not read: {error}"
          ));
        }
      }
      None => problems.push(format!("Weapon section [{name}] visual '{visual}' is not found")),
    }
  }

  /// The HUD item visual resolves, and every motion bank it links resolves and reads.
  fn verify_hud(&self, name: &str, hud_name: &str, hud: &Section, problems: &mut Vec<String>) {
    let visual: &str = hud.get("item_visual").unwrap_or_default();

    let Some(path) = self.find(XrayAssetType::Ogf, visual) else {
      problems.push(format!(
        "Weapon section [{name}] hud section [{hud_name}] item visual '{visual}' is not found"
      ));

      return;
    };

    let refs: Vec<String> = match GamedataVisualMotions::read_refs(self.project, &path) {
      Ok(Some(refs)) => refs,
      Ok(None) => {
        problems.push(format!(
          "Weapon section [{name}] item visual '{visual}' has no motion refs"
        ));

        return;
      }
      Err(error) => {
        problems.push(format!(
          "Weapon section [{name}] item visual '{visual}' motion refs do not read: {error}"
        ));

        return;
      }
    };

    for motion_ref in &refs {
      let banks: BTreeSet<String> = GamedataVisualMotions::resolve_banks(self.project, motion_ref).unwrap_or_default();

      if banks.is_empty() {
        problems.push(format!(
          "Weapon section [{name}] item visual '{visual}' names missing motion bank '{motion_ref}'"
        ));
      }

      for bank in banks {
        if let Err(error) = GamedataVisualMotions::read_names(self.project, &bank) {
          problems.push(format!(
            "Weapon section [{name}] motion bank '{bank}' does not read: {error}"
          ));
        }
      }
    }
  }

  /// A launcher-capable weapon that defines `anm_bore` keeps the launcher bores the engine branches into.
  fn verify_bores(name: &str, section: &Section, hud: &Section, problems: &mut Vec<String>) {
    // Only weapons that can carry a launcher ever reach the grenade launcher bore branch, and one without any bore never
    // enters the state at all.
    if section.get("grenade_launcher_status").is_none_or(|it| it.trim() == "0") || hud.get("anm_bore").is_none() {
      return;
    }

    for required in Self::LAUNCHER_BORES {
      if hud.get(required).is_none() {
        problems.push(format!(
          "Weapon section [{name}] supports a grenade launcher and defines anm_bore, but its hud section has no {required}, and the engine has no fallback for it"
        ));
      }
    }
  }

  /// Every sound the section names resolves, a layered sound section keeping its numbering and naming sounds that
  /// resolve too.
  fn verify_sounds(&self, ltx: &Ltx, name: &str, section: &Section, problems: &mut Vec<String>) {
    for required in Self::REQUIRED_SOUNDS {
      if !section.contains_key(required) {
        problems.push(format!(
          "Weapon section [{name}] has no {required}, a sound every weapon plays"
        ));
      }
    }

    for (field, value) in section {
      // Anomaly based scripts use snd_on_take as a pickup-sound category, not an asset reference.
      if !field.starts_with("snd_") || field == "snd_on_take" {
        continue;
      }

      let value: WeaponSoundValue<'_> = WeaponSoundValue::parse(value);

      if value.is_disabled() {
        continue;
      }

      match WeaponSoundSource::classify(ltx, value) {
        WeaponSoundSource::Asset { name: sound } => {
          if self.find(XrayAssetType::Ogg, sound).is_none() {
            problems.push(format!("Weapon section [{name}] {field} names missing sound '{sound}'"));
          }
        }
        WeaponSoundSource::LayeredSection {
          name: layers_name,
          has_parameters,
          section: layers,
        } => {
          if has_parameters {
            problems.push(format!(
              "Weapon section [{name}] {field} gives layered sound section [{layers_name}] a volume or delay, which a layered sound cannot take"
            ));
          }

          for issue in WeaponSoundLayerIssue::find_all(layers) {
            problems.push(format!(
              "Weapon section [{name}] {field}: {}",
              issue.get_message(layers_name)
            ));
          }

          for (layer_field, layer_value) in layers {
            // Metadata fields such as `$scheme` describe the section, they are not sound references.
            if layer_field.starts_with(LTX_SYMBOL_SCHEME) {
              continue;
            }

            let sound: &str = WeaponSoundValue::parse(layer_value).name();

            if self.find(XrayAssetType::Ogg, sound).is_none() {
              problems.push(format!(
                "Weapon section [{name}] {field}: sound layer section [{layers_name}] {layer_field} names missing sound '{sound}'"
              ));
            }
          }
        }
      }
    }
  }

  /// The logical path of the asset of one kind a reference names, or `None` where nothing resolves it. Resolving through
  /// the VFS is the existence check, loose or archived.
  fn find(&self, kind: XrayAssetType, reference: &str) -> Option<String> {
    self
      .project
      .resolve(kind, reference)
      .ok()
      .flatten()
      .map(|location| location.get_logical_path().to_string())
  }
}
