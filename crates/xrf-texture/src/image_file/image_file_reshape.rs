/// How a texture file about to be written differs from the one already at its path.
///
/// Packing is meant to replace a sheet's pixels, not its geometry, so either answer is worth telling whoever runs it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageFileReshape {
  /// The file being replaced could not be read, so its shape is unknown.
  Unreadable { reason: String },
  /// The file being replaced has another canvas or mip chain, given here as it was.
  Reshaped {
    width: u32,
    height: u32,
    mipmap_levels: u32,
  },
}
