use std::path::PathBuf;

use xrf_renderer::RenderCacheStore;

/// The renderer's cache kept in a folder of the application's cache, a file a name, each written whole or not at all.
pub struct DiskCacheStore {
  directory: PathBuf,
}

impl DiskCacheStore {
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

    let written = std::fs::create_dir_all(&self.directory).and_then(|()| xrf_utils::write_file_staged(&path, bytes));

    if let Err(error) = written {
      log::warn!("The renderer's cache '{}' cannot be kept: {error}", path.display());
    }
  }
}
