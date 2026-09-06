use std::time::Instant;

use xrf_error::XrfResult;

use crate::project::meshes::mesh_assets_verification_result::GamedataMeshAssetsVerificationResult;
use crate::project::meshes::mesh_assets_verifier::MeshAssetsVerifier;
use crate::project::meshes::shader_library_verifier::ShaderLibraryVerifier;
use crate::project::meshes::verify_meshes_result::GamedataMeshesVerificationResult;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions};

pub(crate) struct MeshesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> MeshesVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataMeshesVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify meshes:");

    let started_at: Instant = Instant::now();

    let shader_library = ShaderLibraryVerifier::new(self.project).verify();
    let mesh_assets: GamedataMeshAssetsVerificationResult = match shader_library.library() {
      Some(shader_library) => MeshAssetsVerifier::new(self.project, self.options, shader_library).verify()?,
      None => GamedataMeshAssetsVerificationResult {
        is_skipped: true,
        ..Default::default()
      },
    };

    let result: GamedataMeshesVerificationResult =
      GamedataMeshesVerificationResult::from_checks(started_at.elapsed(), shader_library, mesh_assets);

    xrf_output::info!(
      self.options.output,
      "Verified gamedata meshes in {}, {}",
      xrf_utils::format_duration(result.duration),
      result.get_failure_message()
    );

    self.options.job.check_cancelled()?;

    Ok(result)
  }
}
