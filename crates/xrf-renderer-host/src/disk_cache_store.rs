use xrf_cache::{AppCache, AppCacheStore};
use xrf_renderer::RenderCacheStore;

/// The renderer's cache as one client of an application's caches, which mark what it reads used and tidy what it stops
/// reading.
pub struct DiskCacheStore {
  store: AppCacheStore,
}

impl DiskCacheStore {
  /// The client's name, its folder within the caches: every host keeps the driver's compiled pipelines under it.
  pub const CLIENT: &'static str = "renderer";

  /// The renderer's cache within `cache`.
  pub fn new(cache: &AppCache) -> Self {
    Self {
      store: cache.get_store(Self::CLIENT),
    }
  }
}

impl RenderCacheStore for DiskCacheStore {
  fn read_cached(&self, name: &str) -> Option<Vec<u8>> {
    self.store.read(name)
  }

  fn write_cached(&self, name: &str, bytes: &[u8]) {
    if let Err(error) = self.store.write(name, bytes) {
      log::warn!("The renderer's cache '{name}' cannot be kept: {error}");
    }
  }
}
