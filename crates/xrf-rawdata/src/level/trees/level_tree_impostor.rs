use xrf_dds::RgbaImage;
use xrf_error::XrfResult;

use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;
use crate::level::trees::level_lod_strip::LevelLodStrip;

/// One object's impostor strips, as the Build loads them (`lod\lod_<object>` and its `_nm`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LevelTreeImpostor {
  pub(crate) color: RgbaImage,
  pub(crate) normal: RgbaImage,
}

impl LevelTreeImpostor {
  /// The strips as the textures the Build loads for the model of `reference`, encoded as their atlas
  /// ([`LevelLodStrip::encode`]). They are marked the model's own; a caller naming them for a library tree marks them
  /// [`LevelSceneTextureFile::is_library`].
  ///
  /// # Errors
  ///
  /// Returns an error when a strip does not encode.
  pub(crate) fn into_textures(self, reference: &str) -> XrfResult<[LevelSceneTextureFile; 2]> {
    let name: String = LevelLodStrip::get_texture_name(reference);

    Ok([
      LevelSceneTextureFile {
        bytes: LevelLodStrip::encode(&self.color)?,
        name: name.clone(),
        is_library: false,
      },
      LevelSceneTextureFile {
        bytes: LevelLodStrip::encode(&self.normal)?,
        name: name + LevelLodStrip::NORMAL_SUFFIX,
        is_library: false,
      },
    ])
  }
}
