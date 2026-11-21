use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector2d;

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
  pub texture_coordinates: [Vector2d; 3],
  pub reserved: u16,
  /// The bytes closing the record after `reserved`.
  pub trailing_padding: u16,
}

impl BuildCformRaycastFace {
  /// Bytes one record occupies, padding included.
  pub const SERIALIZED_SIZE: u64 = 36;
}

impl ChunkReadWrite for BuildCformRaycastFace {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let material: u16 = reader.read_u16()?;
    let material_padding: u16 = reader.read_u16()?;
    let game_material: u32 = reader.read_u32()?;
    let mut texture_coordinates: [Vector2d; 3] = [Vector2d::ZERO; 3];

    for coordinate in &mut texture_coordinates {
      *coordinate = reader.read_xr()?;
    }

    Ok(Self {
      material,
      material_padding,
      game_material,
      texture_coordinates,
      reserved: reader.read_u16()?,
      trailing_padding: reader.read_u16()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16(self.material)?;
    writer.write_u16(self.material_padding)?;
    writer.write_u32(self.game_material)?;

    for coordinate in &self.texture_coordinates {
      writer.write_xr(coordinate)?;
    }

    writer.write_u16(self.reserved)?;
    writer.write_u16(self.trailing_padding)?;

    Ok(())
  }
}
