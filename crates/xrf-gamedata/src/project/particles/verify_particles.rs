use std::time::{Duration, Instant};

use rayon::prelude::*;
use xrf_db::{ParticlesFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::particles::verify_particles_result::GamedataParticlesVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  pub fn verify_particles(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataParticlesVerificationResult> {
    xrf_output::heading!(options.output, "Verify particles:");

    let started_at: Instant = Instant::now();
    // Enumerated through the VFS, so an installation's archived libraries are verified too.
    let particle_paths: Vec<String> = self
      .entries_with_suffix("particles.xr")?
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let checked_particle_files_count: u32 = u32::try_from(particle_paths.len())
      .map_err(|_| XrfError::new_verify_error("Particle library count exceeds the supported result range"))?;

    let particle_findings: Vec<Vec<Finding>> = particle_paths
      .par_iter()
      .map(|path| {
        xrf_output::verbose!(options.output, "Verify particles file: {}", path);

        match self.read_parsed(AssetType::XrPack, path, |chunk| {
          ParticlesFile::read_from_chunk::<XRayByteOrder, _>(chunk)
        }) {
          Ok(particles_file) => {
            let particle_findings: Vec<Finding> = self.verify_particle(options, &particles_file, path);

            if !particle_findings.is_empty() {
              xrf_output::info!(options.output, "Particle library is invalid: {}", path);
            }

            particle_findings
          }
          Err(error) => {
            xrf_output::info!(options.output, "Failed to read particle library '{}': {}", path, error);

            vec![GamedataFindingFactory::for_asset(
              GamedataVerificationRule::ParticlesLibrary,
              path,
              format!("Failed to read particle library: {error}"),
            )]
          }
        }
      })
      .collect();

    let duration: Duration = started_at.elapsed();
    let invalid_particle_files_count: u32 =
      u32::try_from(particle_findings.iter().filter(|findings| !findings.is_empty()).count())
        .map_err(|_| XrfError::new_verify_error("Invalid particle library count exceeds the supported result range"))?;

    let mut findings: Vec<Finding> = particle_findings.into_iter().flatten().collect();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    xrf_output::info!(
      options.output,
      "Verified gamedata particle files in {}, {}/{} valid",
      xrf_utils::format_duration(duration),
      checked_particle_files_count - invalid_particle_files_count,
      checked_particle_files_count
    );

    Ok(GamedataParticlesVerificationResult {
      duration,
      checked_particle_files_count,
      findings,
      invalid_particle_files_count,
    })
  }

  pub fn verify_particle(
    &self,
    options: &GamedataProjectVerifyOptions,
    particles_file: &ParticlesFile,
    particle_library_path: &str,
  ) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    for particle in &particles_file.effects.effects {
      xrf_output::verbose!(options.output, "Verify particle: {}", particle.name);

      for texture_relative_path in particle.sprite.texture_name.split(",") {
        if let Some(texture) = self
          .dds_texture(texture_relative_path)
          .ok()
          .flatten()
          .map(|asset| asset.get_logical_path().to_string())
        {
          match self.verify_texture_by_path(options, &texture) {
            Ok(result) => {
              if !result {
                findings.push(GamedataFindingFactory::for_asset(
                  GamedataVerificationRule::ParticlesTexture,
                  &texture,
                  format!("Particle effect '{}' references an invalid texture", particle.name),
                ));
              }
            }
            Err(error) => {
              findings.push(GamedataFindingFactory::for_asset(
                GamedataVerificationRule::ParticlesTexture,
                &texture,
                format!(
                  "Failed to verify texture for particle effect '{}': {error}",
                  particle.name
                ),
              ));
            }
          }
        } else {
          findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::ParticlesTexture,
            particle_library_path,
            format!(
              "Particle effect '{}' references missing texture '{}'",
              particle.name, texture_relative_path
            ),
          ));
        }
      }
    }

    findings
  }
}
