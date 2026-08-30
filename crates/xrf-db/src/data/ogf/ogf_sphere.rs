use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfSphere {
  pub position: Vector3d,
  pub radius: f32,
}

impl ChunkReadWrite for OgfSphere {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      radius: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_f32::<T>(self.radius)?;

    Ok(())
  }
}
