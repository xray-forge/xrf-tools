use serde::{Deserialize, Serialize};

/// What kind of texture a descriptor describes, `STextureParams::ETType` (`ETextureParams.h`).
///
/// The gate on everything else in the file: `CTextureDescrMngr::LoadTHM` (`TextureDescrManager.cpp`) takes the bump,
/// detail and material of a descriptor only when its type is [`Self::Image`], [`Self::NormalMap`] or
/// [`Self::Terrain`]. A cube map or a bump map descriptor contributes nothing, however complete its bump chunk is.
///
/// Absent from a file, the engine's zeroed default is [`Self::Image`], which qualifies.
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

  /// Whether the engine reads the rest of a descriptor of this type at all.
  pub const fn is_described_by_engine(self) -> bool {
    matches!(self, Self::Image | Self::NormalMap | Self::Terrain)
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
