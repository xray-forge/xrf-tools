use std::path::PathBuf;

use image::RgbaImage;
use xrf_dds::Quality;
use xrf_thm::ThmFile;

use crate::build::build_texture_normal_map::BuildTextureNormalMap;

/// What a texture is rebuilt from.
pub struct BuildTextureOptions {
  /// Where the texture is written, which is the `.dds` beside the descriptor; a bump descriptor's companion is written
  /// beside it with `#` after its name.
  pub destination: PathBuf,
  /// Pixels to encode, from whatever image the author is building out of; for a bump descriptor, the height in its
  /// colour and the gloss in its alpha.
  pub source: RgbaImage,
  /// The descriptor that says how, read as a recipe by [`crate::BuildTextureRecipe`].
  pub descriptor: ThmFile,
  /// The external normal map a bump descriptor names, which only its caller can find; none for any other.
  pub normal_map: Option<BuildTextureNormalMap>,
  pub quality: Quality,
}
