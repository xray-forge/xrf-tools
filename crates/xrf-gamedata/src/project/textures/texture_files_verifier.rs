use xrf_dds::DdsFile;
use xrf_error::XrfResult;
use xrf_vfs::XrayAssetType as AssetType;

use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::textures::texture_files_verification_result::GamedataTextureFilesVerificationResult;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Every `.dds` the project holds reads as a texture the engine can upload.
pub(crate) struct TextureFilesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> TextureFilesVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataTextureFilesVerificationResult> {
    self.options.job.check_cancelled()?;

    let texture_paths: Vec<String> = self
      .project
      .entries_of_type(AssetType::Dds)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let outcome: GamedataAssetSweepOutcome = GamedataAssetSweep::new(self.options)
      .run(&texture_paths, |options, path| {
        self.verify_texture(options, path).into_iter().collect()
      })?;

    self.options.job.check_cancelled()?;

    Ok(GamedataTextureFilesVerificationResult {
      checked_textures_count: outcome.checked_count,
      findings: outcome.findings,
      invalid_textures_count: outcome.invalid_count,
    })
  }

  /// Whether one texture reads as a DDS the engine uploads, addressed by its logical path and read through the VFS, so
  /// an archived texture is inspected rather than skipped. Shared with the particles check, which asks the same of the
  /// textures an effect names.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture does not read as a DDS.
  pub(crate) fn is_xray_compatible(project: &GamedataProject, path: &str) -> XrfResult<bool> {
    Ok(DdsFile::read_from_bytes(&project.read_bytes(path)?)?.is_xray_compatible())
  }

  fn verify_texture(&self, options: &GamedataProjectVerifyOptions, path: &str) -> Option<Finding> {
    xrf_output::verbose!(options.output, "Verify texture: {path}");

    match Self::is_xray_compatible(self.project, path) {
      Ok(true) => None,
      Ok(false) => {
        xrf_output::info!(options.output, "Texture is not valid: {path}");

        Some(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::TexturesValidation,
          path,
          "Texture uses an unsupported format",
        ))
      }
      Err(error) => {
        xrf_output::info!(options.output, "Texture verification failed: {path} - {error}");

        Some(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::TexturesRead,
          path,
          error.to_string(),
        ))
      }
    }
  }
}
