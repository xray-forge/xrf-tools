use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlPair};
use xrf_ltx::{Ltx, LtxProject};
use xrf_spawn::{AlifeObjectInherited, SpawnFile, XRayByteOrder};
use xrf_utils::format_path;

use crate::project::gamedata_check::GamedataCheck;
use crate::project::materials::verify_materials_result::GamedataMaterialsVerificationResult;
use crate::{
  GamedataCheckResult, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule,
};

/// Verifies the game material library against what the engine asserts on while loading it, and every name the engine,
/// configs and spawn objects resolve against it.
pub(crate) struct MaterialsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for MaterialsVerifier<'a> {
  type Result = GamedataMaterialsVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataMaterialsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify materials:");

    let started_at: Instant = Instant::now();
    let mut result: GamedataMaterialsVerificationResult = GamedataMaterialsVerificationResult::default();

    match self.project.game_materials() {
      Ok(library) => {
        result.is_library_read = true;
        result.materials_count = library.materials.len() as u32;
        result.pairs_count = library.pairs.len() as u32;

        self.verify_engine_materials(&library, &mut result);
        self.verify_pairs(&library, &mut result);
        self.verify_marks(&library, &mut result);
        self.verify_configs(&library, &mut result);
        self.verify_spawns(&library, &mut result);
      }
      Err(error) => {
        xrf_output::error!(self.options.output, "Game material library was not read: {error}");

        result.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::MaterialsLibrary,
          GameMtlFile::FILE_NAME,
          format!("Failed to read game material library: {error}"),
        ));
      }
    }

    result
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);
    result.duration = started_at.elapsed();

    xrf_output::info!(
      self.options.output,
      "Verified gamedata materials in {}, {}",
      xrf_utils::format_duration(result.duration),
      result.get_summary()
    );

    self.options.job.check_cancelled()?;

    Ok(result)
  }
}

impl MaterialsVerifier<'_> {
  /// Materials the engine looks up by names of its own, and whether it asserts the material is dynamic: `default` in
  /// `Level_load.cpp` and `material_manager.cpp`, `default_object` in `ModelPool.cpp`, the small box in
  /// `PHGeometryOwner.cpp`, and `WEAPON_MATERIAL_NAME` and `KNIFE_MATERIAL_NAME`. A name missing from the library reads
  /// as an index past its end in a release build.
  const ENGINE_MATERIALS: [(&'static str, bool); 5] = [
    ("default", false),
    ("default_object", true),
    ("objects\\small_box", false),
    ("objects\\bullet", false),
    ("objects\\knife", false),
  ];

  /// The key `CMaterialManager::Load` reads an actor's or a monster's material from.
  const MATERIAL_KEY: &'static str = "material";

  fn verify_engine_materials(&self, library: &GameMtlFile, result: &mut GamedataMaterialsVerificationResult) {
    for (name, is_dynamic_required) in Self::ENGINE_MATERIALS {
      result.checked_references_count += 1;

      let message: String = match library.find_material_by_name(name) {
        None => format!("The engine looks up material '{name}', which the library does not define"),
        Some(material) if is_dynamic_required && !material.is_dynamic() => {
          format!("The engine asserts material '{name}' is dynamic, and the library defines it as static")
        }
        Some(_) => continue,
      };

      result.invalid_references_count += 1;
      result.findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::MaterialsEngine,
        GameMtlFile::FILE_NAME,
        message,
      ));
    }
  }

  /// `CGameMtlLibrary::Load` resolves both materials of every pair, reading past its table for a number it does not
  /// hold, and `SGameMtlPair::Load` asserts on a list longer than it loads.
  fn verify_pairs(&self, library: &GameMtlFile, result: &mut GamedataMaterialsVerificationResult) {
    for pair in &library.pairs {
      if self.options.job.is_cancelled() {
        return;
      }

      for material in [pair.material_a, pair.material_b] {
        result.checked_references_count += 1;

        if library.find_material(material).is_none() {
          result.invalid_references_count += 1;
          result.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::MaterialsPair,
            GameMtlFile::FILE_NAME,
            format!(
              "Pair {} names material {material}, which the library does not define",
              library.get_pair_label(pair)
            ),
          ));
        }
      }

      let lists: [(&str, &str, usize); 5] = [
        ("breaking sounds", &pair.breaking_sounds, GameMtlPair::MAX_SOUNDS),
        ("step sounds", &pair.step_sounds, GameMtlPair::MAX_SOUNDS),
        ("collide sounds", &pair.collide_sounds, GameMtlPair::MAX_SOUNDS),
        ("collide particles", &pair.collide_particles, GameMtlPair::MAX_EFFECTS),
        ("collide marks", &pair.collide_marks, GameMtlPair::MAX_EFFECTS),
      ];

      for (label, list, limit) in lists {
        let count: usize = GameMtlPair::split_list(list).len();

        if count > limit {
          result.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::MaterialsList,
            GameMtlFile::FILE_NAME,
            format!(
              "Pair {} lists {count} {label}, more than the {limit} the engine loads",
              library.get_pair_label(pair)
            ),
          ));
        }
      }
    }
  }

  /// Each wallmark texture once, however many pairs name it: a missing one is drawn as the renderer's placeholder.
  fn verify_marks(&self, library: &GameMtlFile, result: &mut GamedataMaterialsVerificationResult) {
    for (mark, pairs_count) in library.get_list_entries(|pair| &pair.collide_marks) {
      if self.options.job.is_cancelled() {
        return;
      }

      result.checked_references_count += 1;

      if self.project.dds_texture(mark).ok().flatten().is_none() {
        result.invalid_references_count += 1;
        result.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::MaterialsMark,
          GameMtlFile::FILE_NAME,
          format!("Wallmark texture '{mark}' named by {pairs_count} pairs is missing"),
        ));
      }
    }
  }

  fn verify_configs(&self, library: &GameMtlFile, result: &mut GamedataMaterialsVerificationResult) {
    for path in &self.project.ltx_project.ltx_file_entries {
      if self.options.job.is_cancelled() {
        return;
      }

      if LtxProject::is_ltx_scheme_path(path) {
        continue;
      }

      // Read through the project: its entries are logical paths, which a filesystem read cannot resolve.
      let reported: PathBuf = self.project.ltx_project.path_of(path);

      match self.project.ltx_project.read_resolution(path) {
        Ok(resolved) => Self::verify_ltx(library, &resolved.ltx, &reported, result),
        // Malformed ltx files are reported by the generic ltx check, not this one.
        Err(error) => xrf_output::verbose!(
          self.options.output,
          "Skipping ltx entry in materials check: {} - {}",
          format_path(&reported),
          error
        ),
      }
    }
  }

  fn verify_ltx(library: &GameMtlFile, ltx: &Ltx, path: &Path, result: &mut GamedataMaterialsVerificationResult) {
    for (section_name, section) in ltx.iter() {
      let Some(value) = section.get(Self::MATERIAL_KEY) else {
        continue;
      };

      let name: &str = value.trim();

      result.checked_references_count += 1;

      if library.find_material_by_name(name).is_none() {
        result.invalid_references_count += 1;
        result.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::MaterialsReference,
          path,
          format!(
            "Unknown game material: [{section_name}] {} = {name}",
            Self::MATERIAL_KEY
          ),
        ));
      }
    }
  }

  /// `CClimableObject::net_Spawn` resolves a ladder's material by name.
  fn verify_spawns(&self, library: &GameMtlFile, result: &mut GamedataMaterialsVerificationResult) {
    for spawn_path in &self.project.spawn_paths() {
      if self.options.job.is_cancelled() {
        return;
      }

      let spawn: Arc<SpawnFile> = match self.project.read_parsed(spawn_path, |chunk| {
        SpawnFile::read_from_chunk::<XRayByteOrder, _>(chunk)
      }) {
        Ok(spawn) => spawn,
        // An unreadable spawn file is the spawns check's finding, not this one's.
        Err(error) => {
          xrf_output::verbose!(
            self.options.output,
            "Skipping spawn file in materials check: {spawn_path} - {error}"
          );
          continue;
        }
      };

      for object in &spawn.alife_spawn.objects {
        let AlifeObjectInherited::CseAlifeObjectClimable(climable) = &object.inherited else {
          continue;
        };

        result.checked_references_count += 1;

        if library.find_material_by_name(&climable.game_material).is_none() {
          result.invalid_references_count += 1;
          result.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::MaterialsReference,
            spawn_path,
            format!(
              "Climable object '{}' is made of unknown game material '{}'",
              object.name, climable.game_material
            ),
          ));
        }
      }
    }
  }
}
