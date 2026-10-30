use std::path::{Path, PathBuf};

use xrf_renderer::RenderCacheStore;

/// The renderer's cache kept in a folder of the application's cache, a file a name, each written whole or not at all; a
/// name with folders in it (`impostors/…`) keeps its file in them.
pub struct DiskCacheStore {
  directory: PathBuf,
}

impl DiskCacheStore {
  /// A store in `directory`, made when the first entry is written.
  pub fn new(directory: PathBuf) -> Self {
    Self { directory }
  }
}

impl RenderCacheStore for DiskCacheStore {
  fn read_cached(&self, name: &str) -> Option<Vec<u8>> {
    std::fs::read(self.directory.join(name)).ok()
  }

  fn write_cached(&self, name: &str, bytes: &[u8]) {
    let path: PathBuf = self.directory.join(name);
    let folder: &Path = path.parent().unwrap_or(&self.directory);

    let written = std::fs::create_dir_all(folder).and_then(|()| xrf_utils::StagedFile::write(&path, bytes));

    if let Err(error) = written {
      log::warn!("The renderer's cache '{}' cannot be kept: {error}", path.display());
    }
  }
}
