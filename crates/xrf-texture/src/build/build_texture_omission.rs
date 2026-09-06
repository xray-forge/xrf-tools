use xrf_db::ThmTextureFlag;

/// One thing a descriptor asks for that this build does not do.
///
/// Reported rather than approximated. A recipe field the converter honoured and we do not is a difference between the
/// texture the author authored and the texture they get, and the only honest thing a tool can do about it is say which
/// field, next to that field.
///
/// Every one of these is `issues/0143`, Stage B of the build parity: fade, border and dithering live inside
/// `nvDXTlibMTDLL.lib` and have to be reimplemented from their documented meaning rather than ported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildTextureOmission {
  /// The per-mip fade the SDK's own chain applies for `Advanced`, which is the one filter value that is not a kernel.
  ///
  /// The chain itself is reproduced: `Build32MipLevel` box-averages each level, which is what a box kernel does. Only
  /// the fade it applies on the way down is missing.
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
  /// The descriptor field this omission sits beside, under the name the SDK gives it.
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

  /// Why the build does not do it, in words a person deciding whether to rebuild can act on.
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
