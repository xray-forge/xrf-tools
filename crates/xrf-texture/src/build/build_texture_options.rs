use std::path::PathBuf;

use image::RgbaImage;
use xrf_db::ThmFile;
use xrf_dds::Quality;

/// What a texture is rebuilt from.
pub struct BuildTextureOptions {
  /// Where the texture is written, which is the `.dds` beside the descriptor.
  pub destination: PathBuf,
  /// Pixels to encode, from whatever image the author is building out of.
  pub source: RgbaImage,
  /// The descriptor that says how, read as a recipe by [`crate::BuildTextureRecipe`].
  pub descriptor: ThmFile,
  pub quality: Quality,
}
