use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::field::chunk_field_reader::ChunkFieldReader;
use crate::field::chunk_field_writer::ChunkFieldWriter;
use crate::read_write::chunk_read_write::ChunkReadWrite;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// A typed payload written once through the engine's `NET_Packet` primitives, so it is read and written both as bytes
/// and as fields of their kinds. Its [`ChunkReadWrite`] is its fields over a [`ChunkReader`] and [`ChunkWriter`].
pub trait ChunkFieldReadWrite: Sized {
  /// # Errors
  ///
  /// Returns an error when the reader does not hold the value.
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self>;

  /// # Errors
  ///
  /// Returns an error when the writer refuses a value.
  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult;
}

impl<F: ChunkFieldReadWrite> ChunkReadWrite for F {
  #[inline]
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Self::read_fields::<T, _>(reader)
  }

  #[inline]
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.write_fields::<T, _>(writer)
  }
}
