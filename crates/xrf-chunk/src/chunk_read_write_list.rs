use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::source::chunk_data_source::ChunkDataSource;
use crate::{ChunkReader, ChunkWriter};

/// Reading is generic over the data source; writing is not.
///
/// A chunked asset may arrive as a file to slice or as bytes already in memory, which is what reading one out of a `.db`
/// volume means — an archived entry has no file. Writing stays file-bound because nothing writes into a volume.
pub trait ChunkReadWriteList: Sized {
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>>;
  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, list: &[Self]) -> XrfResult;
}
