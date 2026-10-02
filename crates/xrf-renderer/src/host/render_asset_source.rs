use xrf_error::XrfResult;

/// Where the renderer's files come from: the application reads them, the renderer never opens one.
pub trait RenderAssetSource: Send + Sync + 'static {
  /// The bytes of the texture a surface names by reference, or `None` for a reference the roots hold nothing for.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture exists but cannot be read.
  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>>;
}
