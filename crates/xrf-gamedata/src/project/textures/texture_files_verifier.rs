use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::textures::texture_files_verification_result::GamedataTextureFilesVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

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
    let texture_paths: Vec<String> = self
      .project
      .entries_of_type(AssetType::Dds)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let checked_textures_count: u32 = u32::try_from(texture_paths.len())
      .map_err(|_| XrfError::new_verify_error("Texture count exceeds the supported result range"))?;

    // Textures are read in parallel, so each one logs into its listed position and the sequence
    // releases them in path order rather than in the order the workers finished.
    let sequence: OutputSequence = OutputSequence::new(&self.options.output, texture_paths.len());

    let mut findings: Vec<Finding> = texture_paths
      .par_iter()
      .enumerate()
      .filter_map(|(index, path)| {
        let slot: OutputSlot = sequence.new_slot(index);
        let output: &OutputOptions = slot.get_output();

        xrf_output::verbose!(output, "Verify texture: {path}");

        match self.project.verify_texture_by_path(path) {
          Ok(true) => None,
          Ok(false) => {
            xrf_output::info!(output, "Texture is not valid: {path}");

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::TexturesValidation,
              path,
              "Texture uses an unsupported format",
            ))
          }
          Err(error) => {
            xrf_output::info!(output, "Texture verification failed: {path} - {error}");

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::TexturesRead,
              path,
              error.to_string(),
            ))
          }
        }
      })
      .collect();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    let invalid_textures_count: u32 = u32::try_from(findings.len())
      .map_err(|_| XrfError::new_verify_error("Invalid texture count exceeds the supported result range"))?;

    Ok(GamedataTextureFilesVerificationResult {
      checked_textures_count,
      findings,
      invalid_textures_count,
    })
  }
}
