use serde::{Deserialize, Serialize};

/// Shading pair a texture asks the renderer for, `STextureParams::ETMaterial` (`ETextureParams.h`).
///
/// Each member names two lighting models the deferred renderer blends between; the weight beside it in the chunk says
/// where between them a surface sits. Values outside the four are in the wild - 59 descriptors of the workspace corpus
/// carry a `6` - so a reader keeps what it read rather than folding it onto a name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum ThmMaterial {
  OrenNayarBlin,
  BlinPhong,
  PhongMetal,
  MetalOrenNayar,
  /// A value the SDK has no name for, kept as it was read so a descriptor written back says what it said.
  Unknown(u32),
}

impl ThmMaterial {
  /// Every material the SDK names, in declaration order.
  pub const NAMED: [Self; 4] = [
    Self::OrenNayarBlin,
    Self::BlinPhong,
    Self::PhongMetal,
    Self::MetalOrenNayar,
  ];

  /// Editor token for the material, `tmtl_token` (`ETextureParams.cpp`), or the number for one the SDK has no name
  /// for.
  pub fn label(self) -> String {
    match self {
      Self::OrenNayarBlin => String::from("OrenNayar <-> Blin"),
      Self::BlinPhong => String::from("Blin <-> Phong"),
      Self::PhongMetal => String::from("Phong <-> Metal"),
      Self::MetalOrenNayar => String::from("Metal <-> OrenNayar"),
      Self::Unknown(raw) => raw.to_string(),
    }
  }
}

impl Default for ThmMaterial {
  /// `STextureParams::STextureParams` sets `tmBlin_Phong` (`ETextureParams.h:120`).
  fn default() -> Self {
    Self::BlinPhong
  }
}

impl From<u32> for ThmMaterial {
  fn from(raw: u32) -> Self {
    match raw {
      0 => Self::OrenNayarBlin,
      1 => Self::BlinPhong,
      2 => Self::PhongMetal,
      3 => Self::MetalOrenNayar,
      raw => Self::Unknown(raw),
    }
  }
}

impl From<ThmMaterial> for u32 {
  fn from(material: ThmMaterial) -> Self {
    match material {
      ThmMaterial::OrenNayarBlin => 0,
      ThmMaterial::BlinPhong => 1,
      ThmMaterial::PhongMetal => 2,
      ThmMaterial::MetalOrenNayar => 3,
      ThmMaterial::Unknown(raw) => raw,
    }
  }
}
