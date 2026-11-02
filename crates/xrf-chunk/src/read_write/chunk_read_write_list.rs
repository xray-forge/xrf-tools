use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// A typed payload a format stores as a repeated sequence, read and written as one list.
///
/// The format decides how the list is framed (a count, one child chunk per entry, records up to the end), which is why
/// this is its own trait rather than a blanket implementation over [`crate::ChunkReadWrite`].
pub trait ChunkReadWriteList: Sized {
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>>;

  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, list: &[Self]) -> XrfResult;
}
