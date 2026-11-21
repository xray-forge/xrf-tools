use xrf_error::XrfResult;

use crate::field::chunk_field_reader::ChunkFieldReader;
use crate::field::chunk_field_writer::ChunkFieldWriter;
use crate::read_write::chunk_read_write_optional::ChunkReadWriteOptional;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// A value a format may leave out, framed through the engine's `NET_Packet` primitives (such as a leading presence
/// flag), the optional form of [`crate::ChunkFieldReadWrite`]. Implementors get [`ChunkReadWriteOptional`] over a
/// [`ChunkReader`] and [`ChunkWriter`].
pub trait ChunkFieldReadWriteOptional: Sized {
  /// # Errors
  ///
  /// Returns an error when the reader does not hold the value or its absence.
  fn read_field_optional<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Option<Self>>;

  /// # Errors
  ///
  /// Returns an error when the writer refuses a value.
  fn write_field_optional<W: ChunkFieldWriter>(writer: &mut W, value: Option<&Self>) -> XrfResult;
}

impl<F: ChunkFieldReadWriteOptional> ChunkReadWriteOptional for F {
  #[inline]
  fn read_optional<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Option<Self>> {
    Self::read_field_optional(reader)
  }

  #[inline]
  fn write_optional(writer: &mut ChunkWriter, value: Option<&Self>) -> XrfResult {
    Self::write_field_optional(writer, value)
  }
}
