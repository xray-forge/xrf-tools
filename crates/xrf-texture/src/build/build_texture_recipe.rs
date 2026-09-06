use xrf_db::{ThmFile, ThmFormat, ThmMipFilter, ThmTextureFlag, ThmTextureParamChunk};
use xrf_dds::{DdsMipFilter, DdsMipmaps, ImageFormat};
use xrf_error::{XrfError, XrfResult};

use crate::build::build_texture_omission::BuildTextureOmission;

/// How a descriptor says its texture should be written, as far as this build can carry it out.
///
/// The whole decision, taken from the descriptor alone and before a single pixel is touched, so that a surface can
/// show what a rebuild would do - and what it would leave out - without doing it.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildTextureRecipe {
  /// The layout to write, from the descriptor's own format.
  pub format: ImageFormat,
  pub mipmaps: DdsMipmaps,
  /// Recipe fields this build does not honour, in the order the descriptor declares them.
  pub omissions: Vec<BuildTextureOmission>,
}

impl BuildTextureRecipe {
  /// Reads a descriptor as a build recipe.
  ///
  /// # Errors
  ///
  /// Returns an error when the descriptor carries no texture params, or when it names a format this build cannot
  /// write. A format is refused rather than substituted: writing BC3 where the author asked for `A8` would produce a
  /// texture that loads and is not the one they described.
  pub fn of(file: &ThmFile) -> XrfResult<Self> {
    let param: &ThmTextureParamChunk = file.texture_param.as_ref().ok_or_else(|| {
      XrfError::new_texture_processing_error("Cannot build from a descriptor carrying no texture parameters")
    })?;

    Ok(Self {
      format: Self::to_image_format(param.format)?,
      mipmaps: Self::to_mipmaps(param),
      omissions: Self::to_omissions(param),
    })
  }

  /// The layout the descriptor's format names, for the four this build can write.
  ///
  /// Those four cover 12,254 of the 13,061 descriptors in the workspace corpus. Three formats are refused rather than
  /// approximated, each for its own reason.
  ///
  /// `tfA8` and `tf565`, 274 descriptors between them, are packings `image_dds` has no encoder for at all.
  ///
  /// `tfADXT1`, 533 of them, is the harder one: it is BC1 carrying a one-bit cutout, and `image_dds`'s BC1 encoder
  /// discards alpha outright - a source with alpha alternating inside every block comes back fully opaque. Writing it
  /// anyway would turn every transparent texel of a fence or a leaf solid, which is a visible change rather than a
  /// recipe field left out, so it is refused until an encoder that does punchthrough is available.
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
  ///
  /// `Advanced` is not a kernel and is not treated as one, but it is not unknown either: it selects the SDK's own
  /// chain, and that chain box-averages every level (`Build32MipLevel`, `xrDXT/DXT.cpp`). So the levels come out the
  /// same and only the fade applied on the way down is missing, which is reported rather than guessed at.
  fn to_mipmaps(param: &ThmTextureParamChunk) -> DdsMipmaps {
    if !param.flags.has(ThmTextureFlag::GenerateMipMaps) {
      return DdsMipmaps::Disabled;
    }

    DdsMipmaps::Filtered(match param.mip_filter {
      ThmMipFilter::Advanced => DdsMipFilter::Box,
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
    })
  }

  /// Every recipe field the descriptor sets that this build does not carry out.
  ///
  /// Only fields the descriptor actually asks for are listed, so a texture that asks for nothing beyond a format and
  /// a filter reports nothing and can be rebuilt with confidence.
  fn to_omissions(param: &ThmTextureParamChunk) -> Vec<BuildTextureOmission> {
    let mut omissions: Vec<BuildTextureOmission> = Vec::new();

    if param.flags.has(ThmTextureFlag::GenerateMipMaps) && param.mip_filter == ThmMipFilter::Advanced {
      omissions.push(BuildTextureOmission::AdvancedMipFade);
    }

    for (flag, omission) in [
      (ThmTextureFlag::BinaryAlpha, BuildTextureOmission::BinaryAlpha),
      (ThmTextureFlag::AlphaBorder, BuildTextureOmission::AlphaBorder),
      (ThmTextureFlag::ColorBorder, BuildTextureOmission::ColorBorder),
      (ThmTextureFlag::FadeToColor, BuildTextureOmission::FadeToColor),
      (ThmTextureFlag::FadeToAlpha, BuildTextureOmission::FadeToAlpha),
      (ThmTextureFlag::DitherColor, BuildTextureOmission::DitherColor),
      (
        ThmTextureFlag::DitherEachMipLevel,
        BuildTextureOmission::DitherEachMipLevel,
      ),
    ] {
      if param.flags.has(flag) {
        omissions.push(omission);
      }
    }

    omissions
  }
}
