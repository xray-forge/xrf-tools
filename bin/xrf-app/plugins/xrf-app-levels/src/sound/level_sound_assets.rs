use xrf_app_core::assets::AssetMountState;
use xrf_audio::AudioAssetSource;
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayRoots, XraySearch};

use crate::sound::LevelSoundFiles;

/// Where a shown level's sounds are read from: the roots it was opened in, through the shared mount.
pub(crate) struct LevelSoundAssets {
  assets: AssetMountState,
  roots: XrayRoots,
  /// What tells its sounds apart from another level's: its roots, as its textures are told apart.
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

    match search.find(&LevelSoundFiles::to_path(reference))? {
      Some(asset) => search.read_asset(&asset).map(Some),
      None => Ok(None),
    }
  }
}
