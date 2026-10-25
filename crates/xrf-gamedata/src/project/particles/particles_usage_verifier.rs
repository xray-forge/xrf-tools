use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_gamemtl::GameMtlFile;
use xrf_ltx::{Ltx, LtxProject};
use xrf_particles::ParticlesFile;
use xrf_spawn::{SpawnFile, XRayByteOrder};
use xrf_utils::format_path;

use crate::project::gamedata_check::GamedataCheck;
use crate::project::particles::verify_particles_usage_result::GamedataParticlesUsageVerificationResult;
use crate::{GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// The particle usage check: every effect or group configs, spawn custom data and material pairs name is in a particle
/// library. A reference to a missing particle is fatal at runtime, the engine asserting on spawn, so it fails the check.
pub(crate) struct ParticlesUsageVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for ParticlesUsageVerifier<'a> {
  type Result = GamedataParticlesUsageVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataParticlesUsageVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify particles usage:");

    let started_at: Instant = Instant::now();
    let particle_names: HashSet<String> = self.read_particle_names()?;

    let mut result: GamedataParticlesUsageVerificationResult = GamedataParticlesUsageVerificationResult::default();

    self.verify_configs(&particle_names, &mut result);
    self.verify_spawns(&particle_names, &mut result);
    self.verify_game_materials(&particle_names, &mut result);

    result
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);
    result.duration = started_at.elapsed();

    xrf_output::info!(
      self.options.output,
      "Verified gamedata particles usage in {}, {}/{} valid references, {}/{} spawn files inspected, {} unparsed custom data sections",
      xrf_utils::format_duration(result.duration),
      result.checked_references_count - result.invalid_references_count,
      result.checked_references_count,
      result.checked_spawn_files_count - result.unreadable_spawn_files_count,
      result.checked_spawn_files_count,
      result.unparsed_custom_data_count
    );

    self.options.job.check_cancelled()?;

    Ok(result)
  }
}

impl ParticlesUsageVerifier<'_> {
  /// Values that appear in particle-typed keys but are not particle names.
  const SKIPPED_REFERENCE_VALUES: [&'static str; 7] = ["true", "false", "on", "off", "0", "1", "nil"];

  /// Every effect and group name the particle libraries in the project define.
  fn read_particle_names(&self) -> XrfResult<HashSet<String>> {
    self.options.job.check_cancelled()?;

    let mut names: HashSet<String> = HashSet::new();

    for library in self.project.entries_with_suffix("particles.xr")? {
      self.options.job.check_cancelled()?;

      // Read from the source that answered the enumeration, rather than searching the mounts again by path.
      let mut chunks = self.project.read_resolved_chunks(&library)?;
      let particles_file: ParticlesFile = ParticlesFile::read_from_chunk::<XRayByteOrder, _>(&mut chunks)?;

      names.extend(
        particles_file
          .effects
          .effects
          .iter()
          .map(|effect| Self::normalize_particle_name(&effect.name)),
      );
      names.extend(
        particles_file
          .groups
          .groups
          .iter()
          .map(|group| Self::normalize_particle_name(&group.name)),
      );
    }

    Ok(names)
  }

  fn verify_configs(&self, particle_names: &HashSet<String>, result: &mut GamedataParticlesUsageVerificationResult) {
    for path in &self.project.ltx_project.ltx_file_entries {
      if self.options.job.is_cancelled() {
        break;
      }

      if LtxProject::is_ltx_scheme_path(path) {
        continue;
      }

      // Read through the project: its entries are logical paths, which a filesystem read cannot resolve.
      let reported: PathBuf = self.project.ltx_project.path_of(path);

      match self.project.ltx_project.read_resolution(path) {
        Ok(resolved) => self.verify_ltx(particle_names, &resolved.ltx, &reported, result),
        // Malformed ltx files are reported by the generic ltx check, not this one.
        Err(error) => xrf_output::verbose!(
          self.options.output,
          "Skipping ltx entry in particles usage check: {} - {}",
          format_path(&reported),
          error
        ),
      }
    }
  }

  fn verify_spawns(&self, particle_names: &HashSet<String>, result: &mut GamedataParticlesUsageVerificationResult) {
    for spawn_path in &self.project.spawn_paths() {
      if self.options.job.is_cancelled() {
        break;
      }

      result.checked_spawn_files_count += 1;

      let spawn_file: Arc<SpawnFile> = match self.project.read_parsed(spawn_path, |chunk| {
        SpawnFile::read_from_chunk::<XRayByteOrder, _>(chunk)
      }) {
        Ok(spawn_file) => spawn_file,
        Err(error) => {
          xrf_output::error!(
            self.options.output,
            "Could not inspect spawn file for particle usage: {} - {}",
            spawn_path,
            error
          );

          result.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::ParticlesUsageSpawn,
            spawn_path,
            format!("Could not inspect spawn file for particle usage: {error}"),
          ));
          result.unreadable_spawn_files_count += 1;
          continue;
        }
      };

      for object in &spawn_file.alife_spawn.objects {
        if self.options.job.is_cancelled() {
          break;
        }

        let Some(custom_data) = object.inherited.get_custom_data() else {
          continue;
        };

        if custom_data.trim().is_empty() {
          continue;
        }

        match Ltx::read_from_str(custom_data) {
          Ok(ltx) => self.verify_ltx(particle_names, &ltx, Path::new(spawn_path), result),
          Err(error) => {
            xrf_output::error!(
              self.options.output,
              "Could not parse spawn custom data for particle usage: {} - {}",
              spawn_path,
              error
            );

            result.findings.push(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::ParticlesUsageSpawnCustomData,
              spawn_path,
              format!("Could not parse spawn custom data for particle usage: {error}"),
            ));
            result.unparsed_custom_data_count += 1;
          }
        }
      }
    }
  }

  /// The effects material pairs play where they collide, each once however many pairs name it. Checked only where the
  /// library reads; the materials check reports it otherwise.
  fn verify_game_materials(
    &self,
    particle_names: &HashSet<String>,
    result: &mut GamedataParticlesUsageVerificationResult,
  ) {
    let Ok(library) = self.project.game_materials() else {
      return;
    };

    for (reference, pairs_count) in library.get_list_entries(|pair| &pair.collide_particles) {
      if self.options.job.is_cancelled() {
        break;
      }

      result.checked_references_count += 1;

      if !particle_names.contains(&Self::normalize_particle_name(reference)) {
        xrf_output::error!(
          self.options.output,
          "Unknown particle reference: collide particles of {pairs_count} pairs = {reference} ({})",
          GameMtlFile::FILE_NAME
        );

        result.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ParticlesUsageReference,
          GameMtlFile::FILE_NAME,
          format!("Unknown particle reference: collide particles of {pairs_count} pairs = {reference}"),
        ));
        result.invalid_references_count += 1;
      }
    }
  }

  fn verify_ltx(
    &self,
    particle_names: &HashSet<String>,
    ltx: &Ltx,
    path: &Path,
    result: &mut GamedataParticlesUsageVerificationResult,
  ) {
    for (section_name, section) in ltx.iter() {
      if self.options.job.is_cancelled() {
        break;
      }

      for (key, value) in section.iter() {
        if !Self::is_particle_reference_key(section_name, key) {
          continue;
        }

        for reference in value.split(',') {
          let reference: &str = reference.trim();

          if reference.is_empty() || Self::SKIPPED_REFERENCE_VALUES.contains(&reference) {
            continue;
          }

          result.checked_references_count += 1;

          if !particle_names.contains(&Self::normalize_particle_name(reference)) {
            xrf_output::error!(
              self.options.output,
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

  /// Whether an ltx key is expected to hold a particle effect or group name.
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
