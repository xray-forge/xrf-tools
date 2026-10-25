use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_spawn::XRayByteOrder;

use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::particles::verify_particles_result::GamedataParticlesVerificationResult;
use crate::project::textures::texture_files_verifier::TextureFilesVerifier;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// The particles check: every particle library reads, and every texture its effects draw is one the engine uploads.
pub(crate) struct ParticlesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for ParticlesVerifier<'a> {
  type Result = GamedataParticlesVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataParticlesVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify particles:");

    let started_at: Instant = Instant::now();
    // Enumerated through the VFS, so an installation's archived libraries are verified too.
    let library_paths: Vec<String> = self
      .project
      .entries_with_suffix("particles.xr")?
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let outcome: GamedataAssetSweepOutcome =
      GamedataAssetSweep::new(self.options).run(&library_paths, |options, path| self.verify_library(options, path))?;

    self.options.job.check_cancelled()?;

    let duration: Duration = started_at.elapsed();

    xrf_output::info!(
      self.options.output,
      "Verified gamedata particle files in {}, {}/{} valid",
      xrf_utils::format_duration(duration),
      outcome.checked_count - outcome.invalid_count,
      outcome.checked_count
    );

    Ok(GamedataParticlesVerificationResult {
      duration,
      checked_particle_files_count: outcome.checked_count,
      findings: outcome.findings,
      invalid_particle_files_count: outcome.invalid_count,
    })
  }
}

impl ParticlesVerifier<'_> {
  fn verify_library(&self, options: &GamedataProjectVerifyOptions, path: &str) -> Vec<Finding> {
    xrf_output::verbose!(options.output, "Verify particles file: {}", path);

    match self
      .project
      .read_parsed(path, ParticlesFile::read_from_chunk::<XRayByteOrder, _>)
    {
      Ok(library) => {
        let findings: Vec<Finding> = self.verify_effect_textures(options, &library, path);

        if !findings.is_empty() {
          xrf_output::info!(options.output, "Particle library is invalid: {}", path);
        }

        findings
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
  }

  /// Every texture an effect's sprite draws, each one that is missing reported against the library and each one that is
  /// not a texture the engine uploads against itself.
  fn verify_effect_textures(
    &self,
    options: &GamedataProjectVerifyOptions,
    library: &ParticlesFile,
    library_path: &str,
  ) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    for effect in &library.effects.effects {
      if options.job.is_cancelled() {
        break;
      }

      xrf_output::verbose!(options.output, "Verify particle: {}", effect.name);

      for reference in effect.sprite.texture_name.split(',') {
        let Some(texture) = self
          .project
          .dds_texture(reference)
          .ok()
          .flatten()
          .map(|asset| asset.get_logical_path().to_string())
        else {
          findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::ParticlesTexture,
            library_path,
            format!(
              "Particle effect '{}' references missing texture '{}'",
              effect.name, reference
            ),
          ));
          continue;
        };

        let message: String = match TextureFilesVerifier::is_xray_compatible(self.project, &texture) {
          Ok(true) => continue,
          Ok(false) => format!("Particle effect '{}' references an invalid texture", effect.name),
          Err(error) => format!(
            "Failed to verify texture for particle effect '{}': {error}",
            effect.name
          ),
        };

        findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ParticlesTexture,
          &texture,
          message,
        ));
      }
    }

    findings
  }
}
