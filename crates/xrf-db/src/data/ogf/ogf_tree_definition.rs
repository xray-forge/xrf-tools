use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf::ogf_color::OgfColor;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfTreeDefinition {
  pub tree_xform: [f32; 16],
  pub scale: OgfColor,
  pub bias: OgfColor,
}

impl ChunkReadWrite for OgfTreeDefinition {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}
