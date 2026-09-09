use xrf_db::ThmTextureFlag;

/// A requested descriptor feature the texture builder does not implement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildTextureOmission {
  /// Missing fade in the `Advanced` mip filter. Box downsampling is implemented.
  AdvancedMipFade,
  FadeToColor,
  FadeToAlpha,
  ColorBorder,
  AlphaBorder,
  DitherColor,
  DitherEachMipLevel,
  BinaryAlpha,
}

impl BuildTextureOmission {
  /// SDK field name associated with this omission.
  pub const fn field(self) -> &'static str {
    match self {
      Self::AdvancedMipFade => "mip_filter",
      Self::FadeToColor => ThmTextureFlag::FadeToColor.label(),
      Self::FadeToAlpha => ThmTextureFlag::FadeToAlpha.label(),
      Self::ColorBorder => ThmTextureFlag::ColorBorder.label(),
      Self::AlphaBorder => ThmTextureFlag::AlphaBorder.label(),
      Self::DitherColor => ThmTextureFlag::DitherColor.label(),
      Self::DitherEachMipLevel => ThmTextureFlag::DitherEachMipLevel.label(),
      Self::BinaryAlpha => ThmTextureFlag::BinaryAlpha.label(),
    }
  }

  /// User-facing explanation of the unsupported feature.
  pub const fn reason(self) -> &'static str {
    match self {
      Self::AdvancedMipFade => "the mip chain is reproduced, but the fade the SDK applies to each level is not",
      Self::FadeToColor => "fading a mip chain towards a colour is not implemented",
      Self::FadeToAlpha => "fading a mip chain towards an alpha is not implemented",
      Self::ColorBorder => "drawing a colour border into the edge texels is not implemented",
      Self::AlphaBorder => "drawing an alpha border into the edge texels is not implemented",
      Self::DitherColor => "dithering the colour channels is not implemented",
      Self::DitherEachMipLevel => "dithering each mip level is not implemented",
      Self::BinaryAlpha => {
        "the converter passes a threshold of zero into a closed library, and no corpus descriptor sets the flag, so \
         there is nothing to reproduce and nothing to check against"
      }
    }
  }
}
