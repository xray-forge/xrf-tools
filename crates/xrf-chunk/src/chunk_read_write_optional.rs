use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::source::chunk_data_source::ChunkDataSource;
use crate::{ChunkReader, ChunkWriter};

pub trait ChunkReadWriteOptional: Sized {
  fn read_optional<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Option<Self>>;
  fn write_optional<T: ByteOrder>(writer: &mut ChunkWriter, it: Option<&Self>) -> XrfResult;
}
