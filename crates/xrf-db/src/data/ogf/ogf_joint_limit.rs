use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Range of motion of one joint axis, `SJointLimit` in the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfJointLimit {
  pub limit_from: f32,
  pub limit_to: f32,
  pub spring_factor: f32,
  pub damping_factor: f32,
}

impl OgfJointLimit {
  pub const SIZE: usize = 4 * 4;
}

impl ChunkReadWrite for OgfJointLimit {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      limit_from: reader.read_f32::<T>()?,
      limit_to: reader.read_f32::<T>()?,
      spring_factor: reader.read_f32::<T>()?,
      damping_factor: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.limit_from)?;
    writer.write_f32::<T>(self.limit_to)?;
    writer.write_f32::<T>(self.spring_factor)?;
    writer.write_f32::<T>(self.damping_factor)?;

    Ok(())
  }
}
