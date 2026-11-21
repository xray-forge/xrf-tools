use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector2d;

/// One triangle of the scene's geometry, `b_face` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjFace {
  /// Its corners, by index into the vertices of the chunk or model holding it.
  pub vertices: [u32; 3],
  pub texture_coordinates: [Vector2d; 3],
  /// Its material, by index into the material table.
  pub material: u16,
  /// The two bytes `#pragma pack(4)` puts before the game material, kept so a rewrite gives back what was read.
  pub material_padding: u16,
  /// The game material it is made of, by its id in `gamemtl.xr`.
  pub game_material: u32,
}

impl BuildPrjFace {
  /// Bytes one face occupies, padding included.
  pub const SERIALIZED_SIZE: u64 = 3 * 4 + 3 * 8 + 2 + 2 + 4;
}

impl ChunkReadWrite for BuildPrjFace {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let vertices: [u32; 3] = [reader.read_u32()?, reader.read_u32()?, reader.read_u32()?];
    let mut texture_coordinates: [Vector2d; 3] = [Vector2d::ZERO; 3];

    for coordinate in &mut texture_coordinates {
      *coordinate = reader.read_xr()?;
    }

    Ok(Self {
      vertices,
      texture_coordinates,
      material: reader.read_u16()?,
      material_padding: reader.read_u16()?,
      game_material: reader.read_u32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in self.vertices {
      writer.write_u32(vertex)?;
    }

    for coordinate in &self.texture_coordinates {
      writer.write_xr(coordinate)?;
    }

    writer.write_u16(self.material)?;
    writer.write_u16(self.material_padding)?;
    writer.write_u32(self.game_material)?;

    Ok(())
  }
}
