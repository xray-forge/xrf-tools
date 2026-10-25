use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::source::chunk_data_source::ChunkDataSource;
use crate::{ChunkReader, ChunkWriter};

pub trait ChunkReadWrite: Sized {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self>;
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult;
}
