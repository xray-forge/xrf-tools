use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// How far one axis of a joint turns and how it springs back, `SJointLimit`.
///
/// The range is in radians, `limit.x` to `limit.y` in the engine. The editor stores it as authored; an OGF stores each
/// limit negated and the two swapped, for the physics engine's opposite sense of rotation (`SJointIKData::Export`), so
/// the same type holds either and the format that reads it says which.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonJointLimit {
  pub min: f32,
  pub max: f32,
  pub spring_factor: f32,
  pub damping_factor: f32,
}

impl SkeletonJointLimit {
  /// Four `f32`s.
  pub const SERIALIZED_SIZE: u64 = 16;
}

impl ChunkReadWrite for SkeletonJointLimit {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      min: reader.read_f32::<T>()?,
      max: reader.read_f32::<T>()?,
      spring_factor: reader.read_f32::<T>()?,
      damping_factor: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.min)?;
    writer.write_f32::<T>(self.max)?;
    writer.write_f32::<T>(self.spring_factor)?;
    writer.write_f32::<T>(self.damping_factor)?;

    Ok(())
  }
}
