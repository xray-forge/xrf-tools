use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

/// How far one axis of a joint turns and how it springs back, `SJointLimit`.
///
/// The range is in radians, `limit.x` to `limit.y` in the engine. The editor stores it as authored; an OGF stores
/// each limit negated and the two swapped (`SJointIKData::Export`). The format that reads it says which it holds.
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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      min: reader.read_f32()?,
      max: reader.read_f32()?,
      spring_factor: reader.read_f32()?,
      damping_factor: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32(self.min)?;
    writer.write_f32(self.max)?;
    writer.write_f32(self.spring_factor)?;
    writer.write_f32(self.damping_factor)?;

    Ok(())
  }
}
