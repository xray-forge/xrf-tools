use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodVertex {
  pub v: Vector3d,
  pub t: (f32, f32),
  pub rgb_hemi: u32,
  pub sun: u8,
  pub pad: [u8; 12],
}

impl ChunkReadWrite for OgfLodVertex {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}
