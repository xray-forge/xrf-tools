use xrf_error::XrfResult;

/// Where the renderer's files come from: the application reads them, the renderer never opens one.
pub trait RenderAssetSource: Send + Sync + 'static {
  /// What a texture reference resolves within: two sources stating the same scope read the same file for a reference,
  /// so the renderer loads it once for both, and two stating different ones are never given each other's.
  fn get_texture_scope(&self) -> String;

  /// The bytes of the texture a surface names by reference, or `None` for a reference the roots hold nothing for.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture exists but cannot be read.
  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>>;
}
