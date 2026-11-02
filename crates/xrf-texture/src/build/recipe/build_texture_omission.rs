use xrf_thm::ThmTextureFlag;

/// A descriptor feature the SDK hands nvDXT, a closed library, and the texture builder therefore does not reproduce.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildTextureOmission {
  /// A fade over a chain nvDXT reduces; the `Advanced` filter's own fade is carried out.
  FadeToColor,
  FadeToAlpha,
  ColorBorder,
  AlphaBorder,
  BinaryAlpha,
}

impl BuildTextureOmission {
  /// SDK field name associated with this omission.
  pub const fn field(self) -> &'static str {
    match self {
      Self::FadeToColor => ThmTextureFlag::FadeToColor.label(),
      Self::FadeToAlpha => ThmTextureFlag::FadeToAlpha.label(),
      Self::ColorBorder => ThmTextureFlag::ColorBorder.label(),
      Self::AlphaBorder => ThmTextureFlag::AlphaBorder.label(),
      Self::BinaryAlpha => ThmTextureFlag::BinaryAlpha.label(),
    }
  }

  /// User-facing explanation of the unsupported feature.
  pub const fn reason(self) -> &'static str {
    match self {
      Self::FadeToColor => {
        "fading a chain the mip filter reduces towards a colour happens inside nvDXT, whose code is not available; the \
         'Advanced' filter's fade is reproduced"
      }
      Self::FadeToAlpha => {
        "fading a chain the mip filter reduces towards an alpha happens inside nvDXT, whose code is not available; the \
         'Advanced' filter's fade is reproduced"
      }
      Self::ColorBorder => "drawing a colour border into the edge texels is not implemented",
      Self::AlphaBorder => "drawing an alpha border into the edge texels is not implemented",
      Self::BinaryAlpha => {
        "the converter passes a threshold of zero into a closed library, and no corpus descriptor sets the flag, so \
         there is nothing to reproduce and nothing to check against"
      }
    }
  }
}
