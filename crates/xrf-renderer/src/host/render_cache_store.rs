/// Where the renderer keeps what it learned in one run for the next, which the application reads and writes for it:
/// the driver's compiled pipelines, so a start compiles only what changed.
pub trait RenderCacheStore: Send + Sync + 'static {
  /// The bytes kept under a name, none where none are kept or they cannot be read.
  fn read_cached(&self, name: &str) -> Option<Vec<u8>>;

  /// Keeps bytes under a name, over what was kept before; a write that fails is the store's to report.
  fn write_cached(&self, name: &str, bytes: &[u8]);
}
