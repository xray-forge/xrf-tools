use xrf_error::XrfResult;

use crate::field::chunk_field_reader::ChunkFieldReader;
use crate::field::chunk_field_writer::ChunkFieldWriter;
use crate::read_write::chunk_read_write_list::ChunkReadWriteList;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// A list framed through the engine's `NET_Packet` primitives (a count, then each entry), the list form of
/// [`crate::ChunkFieldReadWrite`]. Implementors get [`ChunkReadWriteList`] over a [`ChunkReader`] and [`ChunkWriter`].
pub trait ChunkFieldReadWriteList: Sized {
  /// # Errors
  ///
  /// Returns an error when the reader does not hold the list.
  fn read_field_list<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Vec<Self>>;

  /// # Errors
  ///
  /// Returns an error when the list does not fit its framing or the writer refuses a value.
  fn write_field_list<W: ChunkFieldWriter>(writer: &mut W, list: &[Self]) -> XrfResult;
}

impl<F: ChunkFieldReadWriteList> ChunkReadWriteList for F {
  #[inline]
  fn read_list<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    Self::read_field_list(reader)
  }

  #[inline]
  fn write_list(writer: &mut ChunkWriter, list: &[Self]) -> XrfResult {
    Self::write_field_list(writer, list)
  }
}
