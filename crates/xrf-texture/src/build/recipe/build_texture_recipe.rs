use xrf_dds::ImageFormat;
use xrf_error::{XrfError, XrfResult};
use xrf_thm::{ThmFile, ThmFormat, ThmTextureFlag, ThmTextureParamChunk, ThmTextureType};

use crate::build::recipe::build_texture_bump_pair::BuildTextureBumpPair;
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
  /// A bump descriptor is `DXTCompressBump`'s, which ignores its format and mip settings for its own DXT5 with a box
  /// chain, so it is read for its pair alone and leaves nothing out.
  ///
  /// # Errors
  ///
  /// Returns an error when the descriptor is of a type the SDK has no name for, or, unless it is a bump descriptor,
  /// carries no texture params or names a format this build cannot write. A format is refused rather than
  /// substituted: writing BC3 where the author asked for `A8` would produce a texture that loads and is not the one
  /// they described.
  pub fn of(file: &ThmFile) -> XrfResult<Self> {
    // `STextureParams` starts zeroed, which is `ttImage`, so a descriptor without a type chunk is an image.
    let layout: BuildTextureLayout = match file.texture_type.unwrap_or_default() {
      ThmTextureType::Image | ThmTextureType::NormalMap | ThmTextureType::Terrain => BuildTextureLayout::Planar,
      ThmTextureType::CubeMap => BuildTextureLayout::CubeStrip,
      ThmTextureType::BumpMap => {
        return Ok(Self {
          layout: BuildTextureLayout::BumpPair(BuildTextureBumpPair::of(file)),
          format: BuildTextureBumpPair::FORMAT,
          mipmaps: BuildTextureMipmaps::Filtered(BuildTextureBumpPair::MIP_FILTER),
          omissions: Vec::new(),
        });
      }
      ThmTextureType::Unknown(raw) => {
        return Err(XrfError::new_texture_processing_error(format!(
          "Cannot build a texture of type {raw}, which the SDK has no name for and refuses"
        )));
      }
    };
    let param: &ThmTextureParamChunk = file.texture_param.as_ref().ok_or_else(|| {
      XrfError::new_texture_processing_error("Cannot build from a descriptor carrying no texture parameters")
    })?;
    let mipmaps: BuildTextureMipmaps = BuildTextureMipmaps::of(param, file.fade_delay);

    Ok(Self {
      layout,
      format: Self::to_image_format(param.format)?,
      omissions: Self::to_omissions(param, &mipmaps),
      mipmaps,
    })
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
