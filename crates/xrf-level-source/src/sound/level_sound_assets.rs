use std::sync::Arc;

use xrf_audio::AudioAssetSource;
use xrf_error::XrfResult;
use xrf_vfs::{XrayRoots, XraySearch, XrayVfsHost};

use crate::sound::LevelSoundFiles;

/// A shown level's sound source: its opened roots, read through the shared VFS host.
pub(crate) struct LevelSoundAssets {
  host: Arc<XrayVfsHost>,
  roots: XrayRoots,
  /// Keeps its sounds apart from another level's, keyed by its roots as its textures are.
  scope: String,
}

impl LevelSoundAssets {
  pub(crate) fn new(host: Arc<XrayVfsHost>, roots: XrayRoots, scope: String) -> Self {
    Self { host, roots, scope }
  }
}

impl AudioAssetSource for LevelSoundAssets {
  fn get_sound_scope(&self) -> &str {
    &self.scope
  }

  fn read_sound(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let search: XraySearch = self.host.search(&self.roots)?;

    match search.find(&LevelSoundFiles::resolve_path(reference))? {
      Some(asset) => search.read_asset(&asset).map(Some),
      None => Ok(None),
    }
  }
}
