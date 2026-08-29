use std::sync::Arc;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use xrf_db::{ParticlesFile, SpawnFile, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxProject};
use xrf_utils::format_path;
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::levels::level_engine_constants::SPAWNS_DIRECTORY;
use crate::project::particles::verify_particles_usage_result::GamedataParticlesUsageVerificationResult;
use crate::{GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Values that appear in particle-typed keys but are not particle names.
const SKIPPED_REFERENCE_VALUES: [&str; 7] = ["true", "false", "on", "off", "0", "1", "nil"];

impl GamedataProject {
  /// Verify that every particle effect/group referenced from configs and spawn custom data
  /// exists in the shipped particles.xr libraries. A reference to a missing particle is fatal
  /// at runtime (engine asserts on spawn), so it is treated as a verification failure.
  pub fn verify_particles_usage(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataParticlesUsageVerificationResult> {
    xrf_output::heading!(options.output, "Verify particles usage:");

    let started_at: Instant = Instant::now();
    let particle_names: HashSet<String> = self.read_particle_names()?;

    let mut result: GamedataParticlesUsageVerificationResult = GamedataParticlesUsageVerificationResult::default();

    self.verify_particles_usage_in_configs(options, &particle_names, &mut result);
    self.verify_particles_usage_in_spawns(options, &particle_names, &mut result);

    result
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);
    result.duration = started_at.elapsed();

    xrf_output::info!(
      options.output,
      "Verified gamedata particles usage in {}, {}/{} valid references, {}/{} spawn files inspected, {} unparsed custom data sections",
      xrf_utils::format_duration(result.duration),
      result.checked_references_count - result.invalid_references_count,
      result.checked_references_count,
      result.checked_spawn_files_count - result.unreadable_spawn_files_count,
      result.checked_spawn_files_count,
      result.unparsed_custom_data_count
    );

    Ok(result)
  }

  /// Collect known particle effect and group names from all particle files in gamedata roots.
  fn read_particle_names(&self) -> XrfResult<HashSet<String>> {
    let mut names: HashSet<String> = HashSet::new();

    for library in self.entries_with_suffix("particles.xr")? {
      // Read from the source that answered the enumeration, rather than searching the mounts again by path.
      let mut chunks = self.read_resolved_chunks(&library)?;
      let particles_file: ParticlesFile = ParticlesFile::read_from_chunk::<XRayByteOrder, _>(&mut chunks)?;

      for effect in &particles_file.effects.effects {
        names.insert(Self::normalize_particle_name(&effect.name));
      }

      for group in &particles_file.groups.groups {
        names.insert(Self::normalize_particle_name(&group.name));
      }
    }

    Ok(names)
  }

  fn verify_particles_usage_in_configs(
    &self,
    options: &GamedataProjectVerifyOptions,
    particle_names: &HashSet<String>,
    result: &mut GamedataParticlesUsageVerificationResult,
  ) {
    for path in &self.ltx_project.ltx_file_entries {
      if LtxProject::is_ltx_scheme_path(path) {
        continue;
      }

      // Read through the project: its entries are logical paths, which a filesystem read cannot resolve.
      let reported: PathBuf = self.ltx_project.path_of(path);

      match self.ltx_project.read_full(path) {
        Ok(ltx) => {
          self.verify_particles_usage_in_ltx(options, particle_names, &ltx, &reported, result);
        }
        Err(error) => {
          // Malformed ltx files are reported by the generic ltx check, not this one.
          xrf_output::verbose!(
            options.output,
            "Skipping ltx entry in particles usage check: {} - {}",
            format_path(&reported),
            error
          );
        }
      }
    }
  }

  fn verify_particles_usage_in_spawns(
    &self,
    options: &GamedataProjectVerifyOptions,
    particle_names: &HashSet<String>,
    result: &mut GamedataParticlesUsageVerificationResult,
  ) {
    let spawn_files: Vec<String> = self
      .entries_of_type(AssetType::Spawn)
      .into_iter()
      .filter(|location| location.get_logical_path().is_under(SPAWNS_DIRECTORY).unwrap_or(false))
      .map(|location| location.get_logical_path().to_string())
      .collect();

    for relative_path in &spawn_files {
      result.checked_spawn_files_count += 1;

      let Some(spawn_path) = self
        .find(relative_path)
        .ok()
        .flatten()
        .map(|location| location.get_logical_path().to_string())
      else {
        xrf_output::error!(
          options.output,
          "Spawn path not found for particle usage check: {relative_path}"
        );

        result.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ParticlesUsageSpawn,
          Path::new(relative_path),
          "Spawn path was not found in gamedata roots",
        ));
        result.unreadable_spawn_files_count += 1;
        continue;
      };

      let spawn_file: Arc<SpawnFile> = match self.read_parsed(AssetType::Spawn, &spawn_path, |chunk| {
        SpawnFile::read_from_chunk::<XRayByteOrder, _>(chunk)
      }) {
        Ok(spawn_file) => spawn_file,
        Err(error) => {
          xrf_output::error!(
            options.output,
            "Could not inspect spawn file for particle usage: {} - {}",
            spawn_path,
            error
          );

          result.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::ParticlesUsageSpawn,
            &spawn_path,
            format!("Could not inspect spawn file for particle usage: {error}"),
          ));
          result.unreadable_spawn_files_count += 1;
          continue;
        }
      };

      for object in &spawn_file.alife_spawn.objects {
        let Some(custom_data) = object.inherited.get_custom_data() else {
          continue;
        };

        if custom_data.trim().is_empty() {
          continue;
        }

        match Ltx::read_from_str(custom_data) {
          Ok(ltx) => {
            self.verify_particles_usage_in_ltx(options, particle_names, &ltx, Path::new(&spawn_path), result);
          }
          Err(error) => {
            xrf_output::error!(
              options.output,
              "Could not parse spawn custom data for particle usage: {} - {}",
              spawn_path,
              error
            );

            result.findings.push(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::ParticlesUsageSpawnCustomData,
              &spawn_path,
              format!("Could not parse spawn custom data for particle usage: {error}"),
            ));
            result.unparsed_custom_data_count += 1;
          }
        }
      }
    }
  }

  fn verify_particles_usage_in_ltx(
    &self,
    options: &GamedataProjectVerifyOptions,
    particle_names: &HashSet<String>,
    ltx: &Ltx,
    path: &Path,
    result: &mut GamedataParticlesUsageVerificationResult,
  ) {
    for (section_name, section) in &ltx.sections {
      for (key, value) in section.iter() {
        if !Self::is_particle_reference_key(section_name, key) {
          continue;
        }

        for reference in value.split(',') {
          let reference: &str = reference.trim();

          if reference.is_empty() || SKIPPED_REFERENCE_VALUES.contains(&reference) {
            continue;
          }

          result.checked_references_count += 1;

          if !particle_names.contains(&Self::normalize_particle_name(reference)) {
            xrf_output::error!(
              options.output,
              "Unknown particle reference: [{section_name}] {key} = {reference} ({})",
              format_path(path)
            );

            result.findings.push(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::ParticlesUsageReference,
              path,
              format!("Unknown particle reference: [{section_name}] {key} = {reference}"),
            ));
            result.invalid_references_count += 1;
          }
        }
      }
    }
  }

  /// Whether ltx key is expected to contain particle effect or group name.
  fn is_particle_reference_key(section_name: &str, key: &str) -> bool {
    if key.starts_with('$') {
      return false;
    }

    if key == "particles" || key.ends_with("_particles") {
      return true;
    }

    key == "name" && (section_name == "sr_particle" || section_name.starts_with("sr_particle@"))
  }

  fn normalize_particle_name(name: &str) -> String {
    name.trim().to_lowercase().replace('/', "\\")
  }
}
