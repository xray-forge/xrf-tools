use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf::ogf_lod_vertex::OgfLodVertex;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodFace {
  pub lod_vertices: [OgfLodVertex; 4],
}

impl ChunkReadWrite for OgfLodFace {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}
