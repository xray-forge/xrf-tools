use image::RgbaImage;
use xrf_dds::{DdsMipChain, DdsMipFilter, DdsMipmaps};
use xrf_error::XrfResult;

use crate::build::recipe::build_texture_advanced_chain::BuildTextureAdvancedChain;

/// What a texture carries below its base level, and who reduces it.
#[derive(Clone, Debug, PartialEq)]
pub enum BuildTextureMipmaps {
  /// The base alone, for a descriptor without `flGenerateMipMaps`.
  Disabled,
  /// A chain the filter reduces from the base, which the SDK leaves to nvDXT.
  Filtered(DdsMipFilter),
  /// The chain the SDK reduces itself for the `Advanced` filter.
  Advanced(BuildTextureAdvancedChain),
}

impl BuildTextureMipmaps {
  /// The levels one face of the texture is written with.
  ///
  /// # Errors
  ///
  /// Returns an error when the face has no pixels, or the `Advanced` chain cannot halve one of its levels.
  pub fn build(&self, face: &RgbaImage) -> XrfResult<DdsMipChain> {
    match self {
      Self::Disabled => DdsMipChain::build(face, DdsMipmaps::Disabled),
      Self::Filtered(filter) => DdsMipChain::build(face, DdsMipmaps::Filtered(*filter)),
      Self::Advanced(chain) => chain.build(face),
    }
  }
}
