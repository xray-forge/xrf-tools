use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// Cylinder collision primitive, `Fcylinder` in the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfCylinder {
  pub center: Vector3d,
  pub direction: Vector3d,
  pub height: f32,
  pub radius: f32,
}

impl ChunkReadWrite for OgfCylinder {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      center: reader.read_xr::<T, _>()?,
      direction: reader.read_xr::<T, _>()?,
      height: reader.read_f32::<T>()?,
      radius: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.center.write::<T>(writer)?;
    self.direction.write::<T>(writer)?;
    writer.write_f32::<T>(self.height)?;
    writer.write_f32::<T>(self.radius)?;

    Ok(())
  }
}
