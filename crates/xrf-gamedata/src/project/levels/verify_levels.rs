use xrf_error::XrfResult;

use crate::project::levels::levels_verifier::LevelsVerifier;
use crate::project::levels::verify_levels_result::GamedataLevelsVerificationResult;
use crate::{GamedataProject, GamedataProjectVerifyOptions};

impl GamedataProject {
  // todo: Level bundles are the largest asset family in gamedata, so they are the natural first
  //   producer of asset usage data.
  pub fn verify_levels(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<GamedataLevelsVerificationResult> {
    LevelsVerifier::new(self, options).verify()
  }
}
