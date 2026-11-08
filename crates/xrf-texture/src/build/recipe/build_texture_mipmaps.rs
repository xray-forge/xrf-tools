use image::RgbaImage;
use xrf_dds::{DdsMipChain, DdsMipFilter, DdsMipmaps};
use xrf_error::XrfResult;
use xrf_thm::{ThmMipFilter, ThmTextureFlag, ThmTextureParamChunk};

use crate::build::recipe::build_texture_advanced_chain::BuildTextureAdvancedChain;

/// What a texture carries below its base level, and who reduces it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildTextureMipmaps {
  /// The base alone, for a descriptor without `flGenerateMipMaps`.
  Disabled,
  /// A chain the filter reduces from the base, which the SDK leaves to nvDXT.
  Filtered(DdsMipFilter),
  /// The chain the SDK reduces itself for the `Advanced` filter.
  Advanced(BuildTextureAdvancedChain),
}

impl BuildTextureMipmaps {
  /// What a texture of `param` carries below its base level, its `Advanced` chain fading after `fade_delay`.
  pub fn of(param: &ThmTextureParamChunk, fade_delay: Option<u8>) -> Self {
    if !param.flags.has(ThmTextureFlag::GenerateMipMaps) {
      return Self::Disabled;
    }

    let filter: DdsMipFilter = match param.mip_filter {
      // Not a filter: `DXTCompressImage` reduces this chain itself and hands nvDXT the levels.
      ThmMipFilter::Advanced => {
        return Self::Advanced(BuildTextureAdvancedChain::of(param, fade_delay));
      }
      ThmMipFilter::Box => DdsMipFilter::Box,
      ThmMipFilter::Point => DdsMipFilter::Point,
      ThmMipFilter::Triangle => DdsMipFilter::Triangle,
      ThmMipFilter::Quadratic => DdsMipFilter::Quadratic,
      ThmMipFilter::Cubic => DdsMipFilter::Cubic,
      ThmMipFilter::Catrom => DdsMipFilter::Catrom,
      ThmMipFilter::Mitchell => DdsMipFilter::Mitchell,
      ThmMipFilter::Gaussian => DdsMipFilter::Gaussian,
      ThmMipFilter::Sinc => DdsMipFilter::Sinc,
      ThmMipFilter::Bessel => DdsMipFilter::Bessel,
      ThmMipFilter::Hanning => DdsMipFilter::Hanning,
      ThmMipFilter::Hamming => DdsMipFilter::Hamming,
      ThmMipFilter::Blackman => DdsMipFilter::Blackman,
      ThmMipFilter::Kaiser => DdsMipFilter::Kaiser,
      // A filter the SDK has no name for is one nvDXT was never told about either: its `switch` has no case for it,
      // so the library kept whatever it defaults to. Box is the closest thing to that this build can state.
      ThmMipFilter::Unknown(_) => DdsMipFilter::Box,
    };

    Self::Filtered(filter)
  }

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
