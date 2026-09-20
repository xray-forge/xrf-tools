use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// A light colour, the engine's `Fcolor`: four floats rather than the three an OGF colour carries.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelLightColor {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}

impl LevelLightColor {
  /// Bytes the colour occupies.
  pub const SERIALIZED_SIZE: u64 = 4 * 4;
}

impl ChunkReadWrite for LevelLightColor {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      r: reader.read_f32::<T>()?,
      g: reader.read_f32::<T>()?,
      b: reader.read_f32::<T>()?,
      a: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.r)?;
    writer.write_f32::<T>(self.g)?;
    writer.write_f32::<T>(self.b)?;
    writer.write_f32::<T>(self.a)?;

    Ok(())
  }
}
