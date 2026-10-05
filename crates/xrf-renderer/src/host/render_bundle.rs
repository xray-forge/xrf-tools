use xrf_error::XrfResult;

/// Where the renderer's bundle is kept in its checkout, which an application packages and a development build reads.
pub const RENDER_BUNDLE_SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

/// The files the renderer ships with beside its code, kept under [`RENDER_BUNDLE_SOURCE`]: the application packages
/// them and reads them, the renderer never opens one.
pub trait RenderBundle: Send + Sync + 'static {
  /// The bytes of a bundled file, by its path under the bundle, such as `smaa/area.png`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the file when it cannot be read.
  fn read_bundled(&self, path: &str) -> XrfResult<Vec<u8>>;
}
