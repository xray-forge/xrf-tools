use xrf_chunk::XRayByteOrder;
use xrf_environment::EnvironmentCatalog;
use xrf_level::DetailModel;
use xrf_material::XrayTextureScope;
use xrf_utils::error_to_string;
use xrf_vfs::XraySearch;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::textures::resolve_reference;

/// What the open level's cycles are read against: the game's catalog, and where the level finds its textures.
pub struct LevelWeatherSource<'a> {
  pub catalog: &'a EnvironmentCatalog,
  pub search: &'a XraySearch,
  pub scope: XrayTextureScope,
}

impl LevelWeatherSource<'_> {
  /// One texture a keyframe names, found as the level finds its own.
  pub fn locate(&self, reference: &str) -> LevelTextureReference {
    LevelTextureReference {
      logical_path: resolve_reference(self.search, &self.scope, reference),
      reference: reference.to_owned(),
    }
  }

  /// A detail model the weather draws, by its logical path.
  ///
  /// # Errors
  ///
  /// Returns why the model is not in the mounted roots or does not read.
  pub fn read_model(&self, logical_path: &str) -> Result<DetailModel, String> {
    let bytes: Vec<u8> = self.search.read(logical_path).map_err(error_to_string)?;

    DetailModel::read_from_bytes::<XRayByteOrder>(bytes).map_err(error_to_string)
  }
}
