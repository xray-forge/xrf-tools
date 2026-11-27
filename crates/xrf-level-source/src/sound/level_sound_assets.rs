use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use xrf_audio::AudioAssetSource;
use xrf_error::XrfResult;
use xrf_vfs::{XrayInputAnswer, XrayRoots, XraySearch, XrayVfsHost};

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

  /// The file's answer in the roots as they stand, hashed: its source and its revision there, which move with an edit
  /// and when the file comes or goes. None where the roots cannot be searched or the reference spells no path.
  fn get_sound_version(&self, reference: &str) -> Option<u64> {
    let search: XraySearch = self
      .host
      .search(&self.roots)
      .inspect_err(|error| log::warn!("The sound '{reference}' cannot be versioned: {error}"))
      .ok()?;
    let answer: XrayInputAnswer = search.answer_file(&LevelSoundFiles::resolve_path(reference))?;
    let mut hasher: DefaultHasher = DefaultHasher::new();

    answer.hash(&mut hasher);

    Some(hasher.finish())
  }

  fn read_sound(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let search: XraySearch = self.host.search(&self.roots)?;

    match search.find(&LevelSoundFiles::resolve_path(reference))? {
      Some(asset) => search.read_asset(&asset).map(Some),
      None => Ok(None),
    }
  }
}
