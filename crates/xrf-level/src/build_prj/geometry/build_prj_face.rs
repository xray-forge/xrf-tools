use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One triangle of the scene's geometry, `b_face` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjFace {
  /// Its corners, by index into the vertices of the chunk or model holding it.
  pub vertices: [u32; 3],
  pub texture_coordinates: [(f32, f32); 3],
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
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let vertices: [u32; 3] = [
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
    ];
    let mut texture_coordinates: [(f32, f32); 3] = [(0.0, 0.0); 3];

    for coordinate in &mut texture_coordinates {
      *coordinate = (reader.read_f32::<T>()?, reader.read_f32::<T>()?);
    }

    Ok(Self {
      vertices,
      texture_coordinates,
      material: reader.read_u16::<T>()?,
      material_padding: reader.read_u16::<T>()?,
      game_material: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in self.vertices {
      writer.write_u32::<T>(vertex)?;
    }

    for (u, v) in self.texture_coordinates {
      writer.write_f32::<T>(u)?;
      writer.write_f32::<T>(v)?;
    }

    writer.write_u16::<T>(self.material)?;
    writer.write_u16::<T>(self.material_padding)?;
    writer.write_u32::<T>(self.game_material)?;

    Ok(())
  }
}
