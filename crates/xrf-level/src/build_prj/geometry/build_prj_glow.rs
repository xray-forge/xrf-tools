use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One glow placed in the scene, `b_glow` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjGlow {
  pub position: Vector3d<f32>,
  pub size: f32,
  /// Bit 0 marks a glow that does not scale with distance.
  pub flags: u32,
  /// Its material, by index into the material table.
  pub material: u32,
}

impl BuildPrjGlow {
  /// Bytes one glow occupies.
  pub const SERIALIZED_SIZE: u64 = 12 + 4 + 4 + 4;
}

impl ChunkReadWrite for BuildPrjGlow {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      size: reader.read_f32::<T>()?,
      flags: reader.read_u32::<T>()?,
      material: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_f32::<T>(self.size)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_u32::<T>(self.material)?;

    Ok(())
  }
}
