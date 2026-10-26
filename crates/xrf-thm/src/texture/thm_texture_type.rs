use serde::{Deserialize, Serialize};

use crate::texture::thm_format::ThmFormat;
use crate::texture::thm_mip_filter::ThmMipFilter;
use crate::texture::thm_texture_flag::ThmTextureFlag;
use crate::texture::thm_texture_flags::ThmTextureFlags;

/// What kind of texture a descriptor describes, `STextureParams::ETType` (`ETextureParams.h`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum ThmTextureType {
  Image,
  CubeMap,
  BumpMap,
  NormalMap,
  Terrain,
  /// A value the SDK has no name for, kept as it was read so a descriptor written back says what it said.
  Unknown(u32),
}

impl ThmTextureType {
  /// Every type the SDK names, in declaration order, for a surface offering them as a choice.
  pub const NAMED: [Self; 5] = [
    Self::Image,
    Self::CubeMap,
    Self::BumpMap,
    Self::NormalMap,
    Self::Terrain,
  ];

  /// The flags a normal map and a terrain texture both lose on taking their type, `STextureParams::OnTypeChange`.
  const CONVERTER_ONLY_FLAGS: ThmTextureFlags = ThmTextureFlags::none()
    .with(ThmTextureFlag::BinaryAlpha, true)
    .with(ThmTextureFlag::AlphaBorder, true)
    .with(ThmTextureFlag::ColorBorder, true)
    .with(ThmTextureFlag::FadeToColor, true)
    .with(ThmTextureFlag::FadeToAlpha, true)
    .with(ThmTextureFlag::DitherColor, true)
    .with(ThmTextureFlag::DitherEachMipLevel, true)
    .with(ThmTextureFlag::BumpDetail, true);

  /// Whether the engine reads the rest of a descriptor of this type at all.
  pub const fn is_described_by_engine(self) -> bool {
    matches!(self, Self::Image | Self::NormalMap | Self::Terrain)
  }

  /// The flags the SDK's editor clears when a descriptor is given this type (`STextureParams::OnTypeChange`), before
  /// it sets [`Self::get_set_flags`].
  pub const fn get_cleared_flags(self) -> ThmTextureFlags {
    match self {
      Self::BumpMap => ThmTextureFlags::none().with(ThmTextureFlag::GenerateMipMaps, true),
      Self::NormalMap => Self::CONVERTER_ONLY_FLAGS.with(ThmTextureFlag::ImplicitLighted, true),
      Self::Terrain => Self::CONVERTER_ONLY_FLAGS.with(ThmTextureFlag::GenerateMipMaps, true),
      Self::Image | Self::CubeMap | Self::Unknown(_) => ThmTextureFlags::none(),
    }
  }

  /// The flags the SDK's editor sets when a descriptor is given this type.
  pub const fn get_set_flags(self) -> ThmTextureFlags {
    match self {
      Self::NormalMap => ThmTextureFlags::none().with(ThmTextureFlag::GenerateMipMaps, true),
      Self::Terrain => ThmTextureFlags::none().with(ThmTextureFlag::ImplicitLighted, true),
      Self::Image | Self::CubeMap | Self::BumpMap | Self::Unknown(_) => ThmTextureFlags::none(),
    }
  }

  /// The format the SDK's editor gives a descriptor of this type, or none where it keeps the descriptor's own.
  pub const fn get_format(self) -> Option<ThmFormat> {
    match self {
      Self::NormalMap => Some(ThmFormat::Rgba),
      Self::Terrain => Some(ThmFormat::Dxt1),
      Self::Image | Self::CubeMap | Self::BumpMap | Self::Unknown(_) => None,
    }
  }

  /// The mip filter the SDK's editor gives a descriptor of this type, or none where it keeps the descriptor's own.
  pub const fn get_mip_filter(self) -> Option<ThmMipFilter> {
    match self {
      Self::NormalMap => Some(ThmMipFilter::Kaiser),
      Self::Image | Self::CubeMap | Self::BumpMap | Self::Terrain | Self::Unknown(_) => None,
    }
  }

  /// Engine token for the type, `ttype_token` (`ETextureParams.cpp:27`), or the number for one it has no name for.
  pub fn label(self) -> String {
    match self {
      Self::Image => String::from("2D Texture"),
      Self::CubeMap => String::from("Cube Map"),
      Self::BumpMap => String::from("Bump Map"),
      Self::NormalMap => String::from("Normal Map"),
      Self::Terrain => String::from("Terrain"),
      Self::Unknown(raw) => raw.to_string(),
    }
  }
}

impl Default for ThmTextureType {
  /// `STextureParams::STextureParams` zeroes the struct, which is [`Self::Image`] (`ETextureParams.h`).
  fn default() -> Self {
    Self::Image
  }
}

impl From<u32> for ThmTextureType {
  fn from(raw: u32) -> Self {
    match raw {
      0 => Self::Image,
      1 => Self::CubeMap,
      2 => Self::BumpMap,
      3 => Self::NormalMap,
      4 => Self::Terrain,
      raw => Self::Unknown(raw),
    }
  }
}

impl From<ThmTextureType> for u32 {
  fn from(texture_type: ThmTextureType) -> Self {
    match texture_type {
      ThmTextureType::Image => 0,
      ThmTextureType::CubeMap => 1,
      ThmTextureType::BumpMap => 2,
      ThmTextureType::NormalMap => 3,
      ThmTextureType::Terrain => 4,
      ThmTextureType::Unknown(raw) => raw,
    }
  }
}
