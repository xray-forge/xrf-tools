use std::time::Instant;

use xrf_error::XrfResult;

use crate::project::textures::texture_bumps_verifier::TextureBumpsVerifier;
use crate::project::textures::texture_files_verifier::TextureFilesVerifier;
use crate::project::textures::verify_textures_result::GamedataTexturesVerificationResult;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions};

/// The textures check: every `.dds` reads as an X-Ray texture, and every `.thm` bump declaration binds what it names.
pub(crate) struct TexturesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> TexturesVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataTexturesVerificationResult> {
    xrf_output::heading!(self.options.output, "Verify textures:");

    let started_at: Instant = Instant::now();

    let texture_files = TextureFilesVerifier::new(self.project, self.options).verify()?;
    let texture_bumps = TextureBumpsVerifier::new(self.project, self.options).verify();

    let result: GamedataTexturesVerificationResult =
      GamedataTexturesVerificationResult::new(started_at.elapsed(), texture_files, texture_bumps);

    xrf_output::info!(
      self.options.output,
      "Verified gamedata textures in {}, {}",
      xrf_utils::format_duration(result.duration),
      result.get_failure_message()
    );

    Ok(result)
  }
}
