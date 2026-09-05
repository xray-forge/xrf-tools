use xrf_dds::DdsFile;
use xrf_error::XrfResult;

use crate::project::textures::textures_verifier::TexturesVerifier;
use crate::project::textures::verify_textures_result::GamedataTexturesVerificationResult;
use crate::{GamedataProject, GamedataProjectVerifyOptions};

impl GamedataProject {
  pub fn verify_textures(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataTexturesVerificationResult> {
    TexturesVerifier::new(self, options).verify()
  }

  /// Whether one texture reads as an X-Ray compatible DDS, addressed by its logical path.
  ///
  /// Reads through the VFS, so an archived texture is inspected rather than skipped. Shared with the particles check,
  /// which asks the same question of the textures an effect names.
  pub(crate) fn verify_texture_by_path(&self, logical_path: &str) -> XrfResult<bool> {
    Ok(DdsFile::read_from_bytes(&self.read_bytes(logical_path)?)?.is_xray_compatible())
  }
}
