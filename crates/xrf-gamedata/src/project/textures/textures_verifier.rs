use std::time::Instant;

use xrf_error::XrfResult;

use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::textures::texture_bumps_verifier::TextureBumpsVerifier;
use crate::project::textures::texture_files_verifier::TextureFilesVerifier;
use crate::project::textures::verify_textures_result::GamedataTexturesVerificationResult;

/// The textures check: every `.dds` reads as an X-Ray texture, and every `.thm` bump declaration binds what it names.
pub(crate) struct TexturesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for TexturesVerifier<'a> {
  type Result = GamedataTexturesVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataTexturesVerificationResult> {
    self.options.job.check_cancelled()?;

    let started_at: Instant = Instant::now();

    let texture_files = TextureFilesVerifier::new(self.project, self.options).verify()?;
    let texture_bumps = TextureBumpsVerifier::new(self.project, self.options).verify();

    let result: GamedataTexturesVerificationResult =
      GamedataTexturesVerificationResult::new(started_at.elapsed(), texture_files, texture_bumps);

    self.options.job.check_cancelled()?;

    Ok(result)
  }
}
