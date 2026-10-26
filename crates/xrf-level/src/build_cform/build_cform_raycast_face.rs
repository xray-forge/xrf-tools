use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// What the light and AI compilers know of one collision triangle's surface, `b_rc_face` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCformRaycastFace {
  /// The face's material, by index into `build.prj`'s material table.
  pub material: u16,
  /// The two bytes `#pragma pack(4)` puts before the game material, kept so a rewrite gives back what was read.
  pub material_padding: u16,
  /// The game material it is made of, by its id in `gamemtl.xr`.
  pub game_material: u32,
  /// The texture coordinates of its three corners.
  pub texture_coordinates: [(f32, f32); 3],
  pub reserved: u16,
  /// The bytes closing the record after `reserved`.
  pub trailing_padding: u16,
}

impl BuildCformRaycastFace {
  /// Bytes one record occupies, padding included.
  pub const SERIALIZED_SIZE: u64 = 36;
}

impl ChunkReadWrite for BuildCformRaycastFace {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let material: u16 = reader.read_u16::<T>()?;
    let material_padding: u16 = reader.read_u16::<T>()?;
    let game_material: u32 = reader.read_u32::<T>()?;
    let mut texture_coordinates: [(f32, f32); 3] = [(0.0, 0.0); 3];

    for coordinate in &mut texture_coordinates {
      *coordinate = (reader.read_f32::<T>()?, reader.read_f32::<T>()?);
    }

    Ok(Self {
      material,
      material_padding,
      game_material,
      texture_coordinates,
      reserved: reader.read_u16::<T>()?,
      trailing_padding: reader.read_u16::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.material)?;
    writer.write_u16::<T>(self.material_padding)?;
    writer.write_u32::<T>(self.game_material)?;

    for (u, v) in self.texture_coordinates {
      writer.write_f32::<T>(u)?;
      writer.write_f32::<T>(v)?;
    }

    writer.write_u16::<T>(self.reserved)?;
    writer.write_u16::<T>(self.trailing_padding)?;

    Ok(())
  }
}
