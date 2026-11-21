use std::time::Duration;

use xrf_report::Finding;

use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;
use crate::project::meshes::mesh_assets_verification_result::GamedataMeshAssetsVerificationResult;
use crate::project::meshes::shader_library_verification_result::GamedataShaderLibraryVerificationResult;

pub(crate) struct GamedataMeshesVerificationResult {
  pub(crate) duration: Duration,
  findings: Vec<Finding>,
  mesh_assets: GamedataMeshAssetsVerificationResult,
  shader_library: GamedataShaderLibraryVerificationResult,
}

impl GamedataMeshesVerificationResult {
  pub(crate) fn from_checks(
    duration: Duration,
    shader_library: GamedataShaderLibraryVerificationResult,
    mesh_assets: GamedataMeshAssetsVerificationResult,
  ) -> Self {
    let mut findings = shader_library.get_findings().to_vec();

    findings.extend_from_slice(mesh_assets.get_findings());

    Self {
      duration,
      findings,
      mesh_assets,
      shader_library,
    }
  }
}

impl GamedataCheckResult for GamedataMeshesVerificationResult {
  fn get_duration(&self) -> Option<Duration> {
    Some(self.duration)
  }

  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::aggregate([self.shader_library.get_status(), self.mesh_assets.get_status()])
  }

  fn get_summary(&self) -> String {
    format!(
      "{}; {}",
      self.shader_library.get_summary(),
      self.mesh_assets.get_summary()
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use xrf_report::Finding;

  use super::GamedataMeshesVerificationResult;
  use crate::project::gamedata_finding_factory::GamedataFindingFactory;
  use crate::project::gamedata_verification_report::GamedataVerificationReport;
  use crate::project::gamedata_verification_rule::GamedataVerificationRule;
  use crate::project::gamedata_verification_status::GamedataVerificationStatus;
  use crate::project::gamedata_verification_type::GamedataVerificationType;
  use crate::project::meshes::mesh_assets_verification_result::GamedataMeshAssetsVerificationResult;
  use crate::project::meshes::shader_library_verification_result::GamedataShaderLibraryVerificationResult;

  #[test]
  fn exposes_mesh_findings_in_reports() {
    let finding: Finding = GamedataFindingFactory::for_asset(
      GamedataVerificationRule::MeshesValidation,
      "meshes/test.ogf",
      "Mesh references missing texture 'textures/test'",
    );
    let mut report: GamedataVerificationReport = GamedataVerificationReport::default();

    report.add_check(
      GamedataVerificationType::Meshes,
      Ok(GamedataMeshesVerificationResult::from_checks(
        Duration::ZERO,
        GamedataShaderLibraryVerificationResult::passed(Default::default()),
        GamedataMeshAssetsVerificationResult {
          checked_meshes_count: 1,
          findings: vec![finding.clone()],
          invalid_meshes_count: 1,
          ..Default::default()
        },
      )),
    );

    assert_eq!(report.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(report.get_checks()[0].get_findings(), [finding]);
  }
}
