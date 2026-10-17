use xrf_error::XrfResult;

/// Where the renderer's files come from: the application reads them, the renderer never opens one.
pub trait RenderAssetSource: Send + Sync + 'static {
  /// What a texture reference resolves within: two sources stating the same scope read the same file for a reference,
  /// so the renderer loads it once for both, and two stating different ones are never given each other's.
  fn get_texture_scope(&self) -> &str;

  /// The bytes of the texture a surface names by reference, or `None` for a reference the roots hold nothing for.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture exists but cannot be read.
  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>>;

  /// Whether the game loads a texture a step coarser than the rest at the same texture quality, as its
  /// `reduce_lod_texture_list` names it by part of its reference.
  fn is_texture_reduced(&self, _reference: &str) -> bool {
    false
  }

  /// Forgets where the textures of these file names (lowercase, without folder or extension) resolved, found or not,
  /// so the next read finds them afresh: their files changed.
  fn forget_textures(&self, _names: &[String]) {}

  /// A stamp of the file a texture reads from, which changes whenever the file does; none where the texture has no
  /// file of its own (one in an archive, or none found).
  fn get_texture_stamp(&self, _reference: &str) -> Option<u64> {
    None
  }
}

/// A texture reference's file name, lowercase, as a change of its file names it: its last part, without its folders.
pub fn get_texture_name(reference: &str) -> String {
  reference.rsplit(['\\', '/']).next().unwrap_or(reference).to_lowercase()
}
