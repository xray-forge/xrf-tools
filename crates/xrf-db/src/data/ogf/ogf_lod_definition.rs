use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf::ogf_lod_face::OgfLodFace;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodDefinition {
  pub lod_faces: [OgfLodFace; 8],
}

impl ChunkReadWrite for OgfLodDefinition {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}
