use std::sync::Arc;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_gamemtl::GameMtlFile;

use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::meshes::mesh_assets_verification_result::GamedataMeshAssetsVerificationResult;
use crate::project::meshes::mesh_assets_verifier::MeshAssetsVerifier;
use crate::project::meshes::shader_library_verifier::ShaderLibraryVerifier;
use crate::project::meshes::verify_meshes_result::GamedataMeshesVerificationResult;

pub(crate) struct MeshesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for MeshesVerifier<'a> {
  type Result = GamedataMeshesVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataMeshesVerificationResult> {
    self.options.job.check_cancelled()?;

    let started_at: Instant = Instant::now();

    let shader_library = ShaderLibraryVerifier::new(self.project).verify();
    // Bones are checked against the material library only where it reads; the materials check reports it otherwise.
    let game_materials: Option<Arc<GameMtlFile>> = self.project.game_materials().ok();
    let mesh_assets: GamedataMeshAssetsVerificationResult = match shader_library.library() {
      Some(shader_library) => {
        MeshAssetsVerifier::new(self.project, self.options, shader_library, game_materials.as_deref()).verify()?
      }
      None => GamedataMeshAssetsVerificationResult {
        is_skipped: true,
        ..Default::default()
      },
    };

    let result: GamedataMeshesVerificationResult =
      GamedataMeshesVerificationResult::from_checks(started_at.elapsed(), shader_library, mesh_assets);

    self.options.job.check_cancelled()?;

    Ok(result)
  }
}
