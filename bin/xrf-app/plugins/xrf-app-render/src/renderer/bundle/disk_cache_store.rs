use xrf_app_core::cache::AppCacheStore;
use xrf_renderer::RenderCacheStore;

/// The renderer's cache as one client of the application's caches, which mark what it reads used and tidy what it
/// stops reading.
pub struct DiskCacheStore {
  store: AppCacheStore,
}

impl DiskCacheStore {
  /// The renderer's cache over the application's `store` for it.
  pub fn new(store: AppCacheStore) -> Self {
    Self { store }
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
