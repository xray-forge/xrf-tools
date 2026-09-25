use serde::{Deserialize, Serialize};

/// One triangle of the collision form, `CDB::TRI` (`xrCDB/xrCDB.h`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCformFace {
  /// Its corners, by index into the form's vertices, in the engine's winding.
  pub vertices: [u32; 3],
  /// The game material it is made of, by the material's id in `gamemtl.xr`.
  pub material: u16,
  /// Whether it casts no shadow.
  pub is_shadow_suppressed: bool,
  /// Whether it takes no wall marks.
  pub is_wallmark_suppressed: bool,
  /// The render sector it lies in.
  pub sector: u16,
}

impl LevelCformFace {
  /// Bytes one face occupies: three vertex indices and a packed word.
  pub const SERIALIZED_SIZE: usize = 16;

  /// Reads one face out of its stored bytes.
  pub fn of(bytes: &[u8; Self::SERIALIZED_SIZE]) -> Self {
    let word = |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let packed: u32 = word(12);

    Self {
      vertices: [word(0), word(4), word(8)],
      material: (packed & 0x3FFF) as u16,
      is_shadow_suppressed: packed & (1 << 14) != 0,
      is_wallmark_suppressed: packed & (1 << 15) != 0,
      sector: (packed >> 16) as u16,
    }
  }
}
