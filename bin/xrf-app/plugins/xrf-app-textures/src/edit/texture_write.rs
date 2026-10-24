use std::path::PathBuf;

use xrf_app_core::assets::FileStamp;
use xrf_texture::TextureEncodingFormat;

/// The texture half a save writes: a held candidate's bytes, bound for the texture its comparison was made of.
pub struct TextureWrite {
  /// The `.dds` the comparison read, which is the only file its encodes may replace.
  pub path: PathBuf,
  /// The stamp the editor read there.
  pub expected: Option<FileStamp>,
  pub format: TextureEncodingFormat,
  pub bytes: Vec<u8>,
}
