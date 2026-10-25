use xrf_error::XrfResult;

use crate::project::materials::materials_verifier::MaterialsVerifier;
use crate::project::materials::verify_materials_result::GamedataMaterialsVerificationResult;
use crate::{GamedataProject, GamedataProjectVerifyOptions};

impl GamedataProject {
  pub fn verify_materials(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataMaterialsVerificationResult> {
    MaterialsVerifier::new(self, options).verify()
  }
}
