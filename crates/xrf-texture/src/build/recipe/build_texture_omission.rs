use xrf_thm::ThmTextureFlag;

/// A descriptor feature the SDK hands nvDXT, a closed library, whose effect no shipped texture shows, so the texture
/// builder does not reproduce it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildTextureOmission {
  ColorBorder,
  AlphaBorder,
  BinaryAlpha,
}

impl BuildTextureOmission {
  /// SDK field name associated with this omission.
  pub const fn field(self) -> &'static str {
    match self {
      Self::ColorBorder => ThmTextureFlag::ColorBorder.label(),
      Self::AlphaBorder => ThmTextureFlag::AlphaBorder.label(),
      Self::BinaryAlpha => ThmTextureFlag::BinaryAlpha.label(),
    }
  }

  /// User-facing explanation of the unsupported feature.
  pub const fn reason(self) -> &'static str {
    match self {
      Self::ColorBorder => {
        "the colour border is drawn inside nvDXT, whose code is not available, and no shipped texture that asks for one \
         shows what it draws, so there is nothing to reproduce it from"
      }
      Self::AlphaBorder => {
        "the alpha border is drawn inside nvDXT, whose code is not available, and no descriptor anywhere sets the flag, \
         so there is nothing to check a guess against"
      }
      Self::BinaryAlpha => {
        "the converter passes a threshold of zero into a closed library, and no corpus descriptor sets the flag, so \
         there is nothing to reproduce and nothing to check against"
      }
    }
  }
}
