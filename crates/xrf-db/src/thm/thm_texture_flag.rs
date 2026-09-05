use serde::{Deserialize, Serialize};

/// One bit of the texture param flag word the SDK has a name for (`ETextureParams.h`).
///
/// Two of the twelve reach the runtime: [`Self::DiffuseDetail`] and [`Self::BumpDetail`] decide whether the detail
/// chunk is applied at all and how (`TextureDescrManager.cpp:175`). The rest are build recipe, consumed by the
/// converter and gone by the time the engine sees a DDS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThmTextureFlag {
  GenerateMipMaps,
  BinaryAlpha,
  AlphaBorder,
  ColorBorder,
  FadeToColor,
  FadeToAlpha,
  DitherColor,
  DitherEachMipLevel,
  DiffuseDetail,
  ImplicitLighted,
  HasAlpha,
  BumpDetail,
}

impl ThmTextureFlag {
  /// Every flag the SDK names, in bit order.
  pub const NAMED: [Self; 12] = [
    Self::GenerateMipMaps,
    Self::BinaryAlpha,
    Self::AlphaBorder,
    Self::ColorBorder,
    Self::FadeToColor,
    Self::FadeToAlpha,
    Self::DitherColor,
    Self::DitherEachMipLevel,
    Self::DiffuseDetail,
    Self::ImplicitLighted,
    Self::HasAlpha,
    Self::BumpDetail,
  ];

  /// The bit this flag occupies in the flag word.
  pub const fn bit(self) -> u32 {
    match self {
      Self::GenerateMipMaps => 1 << 0,
      Self::BinaryAlpha => 1 << 1,
      Self::AlphaBorder => 1 << 4,
      Self::ColorBorder => 1 << 5,
      Self::FadeToColor => 1 << 6,
      Self::FadeToAlpha => 1 << 7,
      Self::DitherColor => 1 << 8,
      Self::DitherEachMipLevel => 1 << 9,
      Self::DiffuseDetail => 1 << 23,
      Self::ImplicitLighted => 1 << 24,
      Self::HasAlpha => 1 << 25,
      Self::BumpDetail => 1 << 26,
    }
  }

  /// SDK identifier for the flag, which is the name an author of a `.thm` would recognise.
  pub const fn label(self) -> &'static str {
    match self {
      Self::GenerateMipMaps => "flGenerateMipMaps",
      Self::BinaryAlpha => "flBinaryAlpha",
      Self::AlphaBorder => "flAlphaBorder",
      Self::ColorBorder => "flColorBorder",
      Self::FadeToColor => "flFadeToColor",
      Self::FadeToAlpha => "flFadeToAlpha",
      Self::DitherColor => "flDitherColor",
      Self::DitherEachMipLevel => "flDitherEachMIPLevel",
      Self::DiffuseDetail => "flDiffuseDetail",
      Self::ImplicitLighted => "flImplicitLighted",
      Self::HasAlpha => "flHasAlpha",
      Self::BumpDetail => "flBumpDetail",
    }
  }
}
