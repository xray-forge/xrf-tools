use xrf_app_core::assets::AssetMountState;
use xrf_audio::AudioAssetSource;
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayRoots, XraySearch};

use crate::sound::LevelSoundFiles;

/// A shown level's sound source: its opened roots, read through the shared mount.
pub(crate) struct LevelSoundAssets {
  assets: AssetMountState,
  roots: XrayRoots,
  /// Keeps its sounds apart from another level's, keyed by its roots as its textures are.
  scope: String,
}

impl LevelSoundAssets {
  pub(crate) fn new(assets: AssetMountState, roots: XrayRoots, scope: String) -> Self {
    Self { assets, roots, scope }
  }
}

impl AudioAssetSource for LevelSoundAssets {
  fn get_sound_scope(&self) -> &str {
    &self.scope
  }

  fn read_sound(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let search: XraySearch = self.assets.search(&self.roots).map_err(XrfError::new_asset_error)?;

    match search.find(&LevelSoundFiles::resolve_path(reference))? {
      Some(asset) => search.read_asset(&asset).map(Some),
      None => Ok(None),
    }
  }
}
