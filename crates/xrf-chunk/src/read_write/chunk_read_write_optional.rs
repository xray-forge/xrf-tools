use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// A typed payload a format may leave out, read back as `None` when absent.
///
/// The format decides what absence looks like on disk, such as a leading presence flag, which is why this is its own
/// trait rather than a blanket implementation over [`crate::ChunkReadWrite`].
pub trait ChunkReadWriteOptional: Sized {
  fn read_optional<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Option<Self>>;

  fn write_optional<T: ByteOrder>(writer: &mut ChunkWriter, it: Option<&Self>) -> XrfResult;
}
