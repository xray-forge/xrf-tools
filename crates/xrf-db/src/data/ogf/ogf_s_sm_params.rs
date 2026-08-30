use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfSSmParams {}

impl ChunkReadWrite for OgfSSmParams {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}
