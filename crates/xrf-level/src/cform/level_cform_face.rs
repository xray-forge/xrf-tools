use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkRecord, XRayBytes};

/// One triangle of the collision form, `CDB::TRI` (`xrCDB/xrCDB.h`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCformFace {
  /// Its corners, by index into the form's vertices, in the engine's winding.
  pub vertices: [u32; 3],
  /// The game material it is made of, by the material's id in `gamemtl.xr`: the level editor writes the id, and the
  /// engine translates it to an index on load (`xrGame/Level_load.cpp`), which is the index `GetMaterialByIdx` then
  /// takes in `DetailManager_Decompress.cpp`.
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

  /// Where the packed word sits in a face, after its three vertex indices.
  const PACKED_OFFSET: usize = 12;

  /// The bits of the packed word holding the game material: fourteen, so an id is below `1 << 14`.
  pub const MATERIAL_MASK: u32 = 0x3FFF;

  /// The game material of a face's bytes, read where they lie, for a reader wanting nothing else of the face.
  pub fn read_material(face: &[u8; Self::SERIALIZED_SIZE]) -> u16 {
    Self::material_of(XRayBytes::read_u32(&face[Self::PACKED_OFFSET..]))
  }

  fn material_of(packed: u32) -> u16 {
    (packed & Self::MATERIAL_MASK) as u16
  }
}

/// Three corners, then the word packing its material, flags and sector.
impl ChunkRecord<{ Self::SERIALIZED_SIZE }> for LevelCformFace {
  #[inline]
  fn decode(bytes: &[u8; Self::SERIALIZED_SIZE]) -> Self {
    let packed: u32 = XRayBytes::read_u32(&bytes[Self::PACKED_OFFSET..]);

    Self {
      vertices: [
        XRayBytes::read_u32(&bytes[0..4]),
        XRayBytes::read_u32(&bytes[4..8]),
        XRayBytes::read_u32(&bytes[8..12]),
      ],
      material: Self::material_of(packed),
      is_shadow_suppressed: packed & (1 << 14) != 0,
      is_wallmark_suppressed: packed & (1 << 15) != 0,
      sector: (packed >> 16) as u16,
    }
  }

  #[inline]
  fn encode(&self) -> [u8; Self::SERIALIZED_SIZE] {
    let mut bytes: [u8; Self::SERIALIZED_SIZE] = [0; Self::SERIALIZED_SIZE];

    for (corner, vertex) in self.vertices.iter().enumerate() {
      XRayBytes::write_u32(&mut bytes[corner * 4..], *vertex);
    }

    XRayBytes::write_u32(
      &mut bytes[Self::PACKED_OFFSET..],
      (u32::from(self.material) & Self::MATERIAL_MASK)
        | (u32::from(self.is_shadow_suppressed) << 14)
        | (u32::from(self.is_wallmark_suppressed) << 15)
        | (u32::from(self.sector) << 16),
    );

    bytes
  }
}
