use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

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
}

impl ChunkReadWrite for LevelCformFace {
  /// Reads one face: its three corners, then the word packing its material, flags and sector.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let vertices: [u32; 3] = [
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
    ];
    let packed: u32 = reader.read_u32::<T>()?;

    Ok(Self {
      vertices,
      material: (packed & 0x3FFF) as u16,
      is_shadow_suppressed: packed & (1 << 14) != 0,
      is_wallmark_suppressed: packed & (1 << 15) != 0,
      sector: (packed >> 16) as u16,
    })
  }

  /// Writes one face in the layout [`Self::read`] reads.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in self.vertices {
      writer.write_u32::<T>(vertex)?;
    }

    writer.write_u32::<T>(
      u32::from(self.material & 0x3FFF)
        | (u32::from(self.is_shadow_suppressed) << 14)
        | (u32::from(self.is_wallmark_suppressed) << 15)
        | (u32::from(self.sector) << 16),
    )?;

    Ok(())
  }
}
