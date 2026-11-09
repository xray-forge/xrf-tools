use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::field::chunk_field_reader::ChunkFieldReader;
use crate::field::chunk_field_writer::ChunkFieldWriter;
use crate::read_write::chunk_read_write_optional::ChunkReadWriteOptional;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// A value a format may leave out, framed through the engine's `NET_Packet` primitives (such as a leading presence
/// flag), read and written both as bytes and as fields. Its [`ChunkReadWriteOptional`] is its fields over a
/// [`ChunkReader`] and [`ChunkWriter`].
pub trait ChunkFieldReadWriteOptional: Sized {
  /// # Errors
  ///
  /// Returns an error when the reader does not hold the value or its absence.
  fn read_field_optional<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Option<Self>>;

  /// # Errors
  ///
  /// Returns an error when the writer refuses a value.
  fn write_field_optional<T: ByteOrder, W: ChunkFieldWriter>(writer: &mut W, value: Option<&Self>) -> XrfResult;
}

impl<F: ChunkFieldReadWriteOptional> ChunkReadWriteOptional for F {
  #[inline]
  fn read_optional<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Option<Self>> {
    Self::read_field_optional::<T, _>(reader)
  }

  #[inline]
  fn write_optional<T: ByteOrder>(writer: &mut ChunkWriter, value: Option<&Self>) -> XrfResult {
    Self::write_field_optional::<T, _>(writer, value)
  }
}
