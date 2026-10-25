use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// A typed payload that reads itself from a chunk and writes itself back.
///
/// Reading is generic over the data source, because a chunked asset arrives either as a file to slice or as bytes
/// already in memory, which is what an entry read out of a `.db` volume is. Writing always goes into a [`ChunkWriter`]
/// buffer, which the caller frames and places.
pub trait ChunkReadWrite: Sized {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self>;

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult;
}
