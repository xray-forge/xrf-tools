use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfBox {
  pub min: Vector3d,
  pub max: Vector3d,
}

impl ChunkReadWrite for OgfBox {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      min: reader.read_xr::<T, _>()?,
      max: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.min)?;
    writer.write_xr::<T, _>(&self.max)?;

    Ok(())
  }
}
