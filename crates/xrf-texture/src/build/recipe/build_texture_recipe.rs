use xrf_dds::{DdsMipFilter, ImageFormat};
use xrf_error::{XrfError, XrfResult};
use xrf_thm::{ThmFile, ThmFormat, ThmMipFilter, ThmTextureFlag, ThmTextureParamChunk, ThmTextureType};

use crate::build::recipe::build_texture_advanced_chain::BuildTextureAdvancedChain;
use crate::build::recipe::build_texture_layout::BuildTextureLayout;
use crate::build::recipe::build_texture_mipmaps::BuildTextureMipmaps;
use crate::build::recipe::build_texture_omission::BuildTextureOmission;

/// How a descriptor says its texture should be written, as far as this build can carry it out: the SDK's
/// `DXTCompress` (`xray/trunk/xrDXT/DXT.cpp`) read as a recipe.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildTextureRecipe {
  pub layout: BuildTextureLayout,
  /// The layout to write, from the descriptor's own format.
  pub format: ImageFormat,
  pub mipmaps: BuildTextureMipmaps,
  /// Recipe fields this build does not honour, in the order the descriptor declares them.
  pub omissions: Vec<BuildTextureOmission>,
}

impl BuildTextureRecipe {
  /// Reads a descriptor as a build recipe.
  ///
  /// # Errors
  ///
  /// Returns an error when the descriptor carries no texture params, is a bump descriptor or of a type the SDK has no
  /// name for, or names a format this build cannot write. A format is refused rather than substituted: writing BC3
  /// where the author asked for `A8` would produce a texture that loads and is not the one they described.
  pub fn of(file: &ThmFile) -> XrfResult<Self> {
    let param: &ThmTextureParamChunk = file.texture_param.as_ref().ok_or_else(|| {
      XrfError::new_texture_processing_error("Cannot build from a descriptor carrying no texture parameters")
    })?;
    let mipmaps: BuildTextureMipmaps = Self::to_mipmaps(param, file.fade_delay);

    Ok(Self {
      // `STextureParams` starts zeroed, which is `ttImage`, so a descriptor without a type chunk is an image.
      layout: Self::to_layout(file.texture_type.unwrap_or_default())?,
      format: Self::to_image_format(param.format)?,
      omissions: Self::to_omissions(param, &mipmaps),
      mipmaps,
    })
  }

  /// The layout `DXTCompress` routes the type to.
  fn to_layout(texture_type: ThmTextureType) -> XrfResult<BuildTextureLayout> {
    match texture_type {
      ThmTextureType::Image | ThmTextureType::NormalMap | ThmTextureType::Terrain => Ok(BuildTextureLayout::Planar),
      ThmTextureType::CubeMap => Ok(BuildTextureLayout::CubeStrip),
      // `DXTCompressBump` writes two files from one packed source, which is a different operation from this one.
      ThmTextureType::BumpMap => Err(XrfError::new_texture_processing_error(
        "A bump descriptor builds the '_bump' and '_bump#' pair, which GenerateBumpProcessor writes, not one texture",
      )),
      ThmTextureType::Unknown(raw) => Err(XrfError::new_texture_processing_error(format!(
        "Cannot build a texture of type {raw}, which the SDK has no name for and refuses"
      ))),
    }
  }

  /// The layout the descriptor's format names, for the four this build can write.
  fn to_image_format(format: ThmFormat) -> XrfResult<ImageFormat> {
    match format {
      // BC1 with no alpha, which is what the descriptor asked for: the encoder discards the source's own.
      ThmFormat::Dxt1 => Ok(ImageFormat::BC1RgbaUnorm),
      ThmFormat::Dxt3 => Ok(ImageFormat::BC2RgbaUnorm),
      ThmFormat::Dxt5 => Ok(ImageFormat::BC3RgbaUnorm),
      ThmFormat::Rgba => Ok(ImageFormat::Rgba8Unorm),
      ThmFormat::Dxt1Alpha => Err(XrfError::new_texture_processing_error(
        "Cannot build a texture in format 'DXT1 Alpha': the encoder available here writes BC1 without its one-bit \
         cutout, so every transparent texel would come back opaque",
      )),
      other => Err(XrfError::new_texture_processing_error(format!(
        "Cannot build a texture in format '{}', which this build has no encoder for",
        other.label()
      ))),
    }
  }

  /// What the texture carries below its base level.
  fn to_mipmaps(param: &ThmTextureParamChunk, fade_delay: Option<u8>) -> BuildTextureMipmaps {
    if !param.flags.has(ThmTextureFlag::GenerateMipMaps) {
      return BuildTextureMipmaps::Disabled;
    }

    let filter: DdsMipFilter = match param.mip_filter {
      // Not a filter: `DXTCompressImage` reduces this chain itself and hands nvDXT the levels.
      ThmMipFilter::Advanced => {
        return BuildTextureMipmaps::Advanced(BuildTextureAdvancedChain::of(param, fade_delay));
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

    BuildTextureMipmaps::Filtered(filter)
  }

  /// Every recipe field the descriptor sets that this build does not carry out.
  ///
  /// The dithering flags are not among them: nvDXT dithers only a conversion to 16 bits, none of the formats this
  /// writes, and its header says even that is disabled (`xray/trunk/xrDXT/nvdxt_options.h`), so a dithered descriptor
  /// was built undithered by the SDK as well. A fade is nvDXT's own only for a chain it reduces; the `Advanced` chain
  /// fades as the SDK does, and a texture without mip maps has nothing to fade.
  fn to_omissions(param: &ThmTextureParamChunk, mipmaps: &BuildTextureMipmaps) -> Vec<BuildTextureOmission> {
    let is_filtered: bool = matches!(mipmaps, BuildTextureMipmaps::Filtered(_));

    [
      (ThmTextureFlag::BinaryAlpha, BuildTextureOmission::BinaryAlpha, true),
      (ThmTextureFlag::AlphaBorder, BuildTextureOmission::AlphaBorder, true),
      (ThmTextureFlag::ColorBorder, BuildTextureOmission::ColorBorder, true),
      (
        ThmTextureFlag::FadeToColor,
        BuildTextureOmission::FadeToColor,
        is_filtered,
      ),
      (
        ThmTextureFlag::FadeToAlpha,
        BuildTextureOmission::FadeToAlpha,
        is_filtered,
      ),
    ]
    .into_iter()
    .filter(|(flag, _, applies)| *applies && param.flags.has(*flag))
    .map(|(_, omission, _)| omission)
    .collect()
  }
}
