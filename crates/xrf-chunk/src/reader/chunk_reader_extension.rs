use std::io::Read;

use byteorder::{ByteOrder, ReadBytesExt};
use xrf_error::{XrfError, XrfResult};

use crate::read_write::chunk_read_write::ChunkReadWrite;
use crate::read_write::chunk_read_write_list::ChunkReadWriteList;
use crate::read_write::chunk_read_write_optional::ChunkReadWriteOptional;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;

impl<D: ChunkDataSource> ChunkReader<D> {
  #[inline]
  pub fn read_xr<T: ByteOrder, C: ChunkReadWrite>(&mut self) -> XrfResult<C> {
    C::read::<T, D>(self)
  }

  #[inline]
  pub fn read_xr_optional<T: ByteOrder, C: ChunkReadWriteOptional>(&mut self) -> XrfResult<Option<C>> {
    C::read_optional::<T, D>(self)
  }

  #[inline]
  pub fn read_xr_list<T: ByteOrder, C: ChunkReadWriteList>(&mut self) -> XrfResult<Vec<C>> {
    C::read_list::<T, D>(self)
  }

  /// Read a chunk whose whole payload is one `u8`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload is shorter, or longer, than the value.
  pub fn read_u8_payload(&mut self) -> XrfResult<u8> {
    self.read_whole_payload("All data should be read in u8 chunk", |reader| Ok(reader.read_u8()?))
  }

  /// Read a chunk whose whole payload is one `u16`; see [`Self::read_u8_payload`] for the errors.
  pub fn read_u16_payload<T: ByteOrder>(&mut self) -> XrfResult<u16> {
    self.read_whole_payload("All data should be read in u16 chunk", |reader| {
      Ok(reader.read_u16::<T>()?)
    })
  }

  /// Read a chunk whose whole payload is one `u32`; see [`Self::read_u8_payload`] for the errors.
  pub fn read_u32_payload<T: ByteOrder>(&mut self) -> XrfResult<u32> {
    self.read_whole_payload("All data should be read in u32 chunk", |reader| {
      Ok(reader.read_u32::<T>()?)
    })
  }

  /// Read a chunk whose whole payload is one `f32`; see [`Self::read_u8_payload`] for the errors.
  pub fn read_f32_payload<T: ByteOrder>(&mut self) -> XrfResult<f32> {
    self.read_whole_payload("All data should be read in f32 chunk", |reader| {
      Ok(reader.read_f32::<T>()?)
    })
  }

  /// Read a chunk whose whole payload is three `f32`s; see [`Self::read_u8_payload`] for the errors.
  pub fn read_f32_vector_payload<T: ByteOrder>(&mut self) -> XrfResult<(f32, f32, f32)> {
    self.read_whole_payload("All data should be read in f32 vector chunk", |reader| {
      Ok((
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
      ))
    })
  }

  /// Read a chunk whose whole payload is one null-terminated string; see [`Self::read_u8_payload`] for the errors.
  pub fn read_w1251_string_payload(&mut self) -> XrfResult<String> {
    self.read_whole_payload("All data should be read in string chunk", Self::read_w1251_string)
  }

  /// Create a vector for a record count this chunk declares, bounded by the bytes remaining in it.
  ///
  /// `min_record_size` counts only the bytes every record consumes unconditionally; see
  /// [`xrf_utils::assert_count_fits`].
  pub fn new_bounded_vec<C>(&self, count: u64, min_record_size: u64, what: &str) -> XrfResult<Vec<C>> {
    xrf_utils::new_bounded_vec(count, self.read_bytes_remain(), min_record_size, what)
  }

  /// Read the rest of the chunk as a packed run of fixed-size records, which is how the engine reads a chunk it casts
  /// onto an array: no count is stored, the size alone says how many there are.
  ///
  /// `what` names the chunk in the errors.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload does not divide into whole `record_size` byte records, a record cannot be read,
  /// or the records read do not end exactly at the chunk end.
  pub fn read_xr_records<T: ByteOrder, C: ChunkReadWrite>(
    &mut self,
    record_size: u64,
    what: &str,
  ) -> XrfResult<Vec<C>> {
    let size: u64 = self.read_bytes_remain();

    if !size.is_multiple_of(record_size) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected {what} of {size} bytes, which does not divide into {record_size} byte records"
      )));
    }

    let count: u64 = size / record_size;
    let mut records: Vec<C> = self.new_bounded_vec(count, record_size, what)?;

    for _ in 0..count {
      records.push(C::read::<T, D>(self)?);
    }

    self.assert_read(&format!("Expect all data to be read from {what}"))?;

    Ok(records)
  }

  /// Read serialized vector from chunk, where u32 count N is followed by N u16 entries.
  pub fn read_u16_vector<T: ByteOrder>(&mut self) -> XrfResult<Vec<u16>> {
    let count: u32 = self.read_u32::<T>()?;
    let mut vector: Vec<u16> = self.new_bounded_vec(count.into(), size_of::<u16>() as u64, "u16 vector")?;

    for _ in 0..count {
      vector.push(self.read_u16::<T>()?)
    }

    Ok(vector)
  }

  /// Read a float the format stored as a `u16` spread across a range, `IReader::r_float_q16`.
  ///
  /// Lossy in both directions, and not corrected: this is the number the engine reads, and a reader that
  /// rounded it to something prettier would disagree with what the game does.
  pub fn read_f32_q16<T: ByteOrder>(&mut self, minimum: f32, maximum: f32) -> XrfResult<f32> {
    let value: u16 = self.read_u16::<T>()?;

    Ok(f32::from(value) * (maximum - minimum) / f32::from(u16::MAX) + minimum)
  }

  /// Read raw bytes.
  ///
  /// # Errors
  ///
  /// Returns an error, before reading anything, when fewer than `count` bytes remain in the chunk.
  pub fn read_bytes(&mut self, count: usize) -> XrfResult<Vec<u8>> {
    if count as u64 > self.read_bytes_remain() {
      return Err(XrfError::new_invalid_error(format!(
        "Requested {count} bytes, but only {} remain in the chunk",
        self.read_bytes_remain()
      )));
    }

    Ok(self.data.read_bytes(count)?)
  }

  /// Read all remaining raw bytes.
  pub fn read_remaining(&mut self) -> XrfResult<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();

    self.read_to_end(&mut buf)?;

    Ok(buf)
  }

  /// Reads one value and proves it was the whole payload.
  fn read_whole_payload<V>(&mut self, message: &str, read: impl FnOnce(&mut Self) -> XrfResult<V>) -> XrfResult<V> {
    let value: V = read(self)?;

    self.assert_read(message)?;

    Ok(value)
  }
}

#[cfg(test)]
mod tests {
  use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
  use xrf_error::XrfResult;

  use crate::XRayByteOrder;
  use crate::read_write::chunk_read_write::ChunkReadWrite;
  use crate::reader::chunk_reader::ChunkReader;
  use crate::source::chunk_data_source::ChunkDataSource;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;
  use crate::writer::chunk_writer::ChunkWriter;

  #[test]
  fn test_read_u16_vector() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_bytes(&[4, 0, 0, 0, 0, 0, 1, 0, 2, 0, 3, 0])?;

    assert_eq!(chunk.read_bytes_remain(), 12, "Expect 12 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(
      chunk.read_u16_vector::<XRayByteOrder>()?,
      vec!(0u16, 1u16, 2u16, 3u16),
      "Expect correctly read vector"
    );
    assert_eq!(chunk.cursor_pos(), 12, "Expect 12 bytes read");

    Ok(())
  }

  #[test]
  fn test_read_bytes() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])?;

    assert_eq!(chunk.read_bytes_remain(), 10, "Expect 10 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(
      chunk.read_bytes(10)?,
      vec!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
      "Expect correctly read raw bytes"
    );
    assert_eq!(chunk.cursor_pos(), 10, "Expect 10 bytes read");

    Ok(())
  }

  #[test]
  fn rejects_byte_count_larger_than_remaining_payload_before_reading() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[1, 2])?;
    let error: String = chunk
      .read_bytes(1_000)
      .expect_err("expect the requested byte count to exceed the payload")
      .to_string();

    assert!(error.contains("Requested 1000 bytes"), "Unexpected error: {error}");
    assert_eq!(chunk.cursor_pos(), 0);

    Ok(())
  }

  #[test]
  fn test_read_remaining() -> XrfResult {
    assert_eq!(ChunkReader::from_bytes(&[0, 1, 2])?.read_remaining()?, vec![0, 1, 2]);
    assert_eq!(ChunkReader::from_bytes(&[0])?.read_remaining()?, vec![0]);
    // Nothing left to read is reached through an empty chunk, since a reader cannot be opened over an empty source.
    let mut empty: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0; 8])?.read_child_by_index(0)?;

    assert_eq!(empty.read_remaining()?, Vec::<u8>::new());

    Ok(())
  }

  #[test]
  fn reads_a_payload_holding_exactly_one_value() -> XrfResult {
    assert_eq!(ChunkReader::from_bytes(&[7])?.read_u8_payload()?, 7);
    assert_eq!(
      ChunkReader::from_bytes(&[1, 0, 0, 0])?.read_u32_payload::<XRayByteOrder>()?,
      1
    );
    assert_eq!(ChunkReader::from_bytes(b"abc\0")?.read_w1251_string_payload()?, "abc");

    Ok(())
  }

  #[test]
  fn refuses_a_payload_with_bytes_left_after_its_value() -> XrfResult {
    let error: String = ChunkReader::from_bytes(&[1, 0, 0, 0, 9])?
      .read_u32_payload::<XRayByteOrder>()
      .expect_err("expect the trailing byte to be refused")
      .to_string();

    assert_eq!(
      error,
      "Chunk is not ended, 1 bytes remain: All data should be read in u32 chunk"
    );

    Ok(())
  }

  #[test]
  fn bounds_a_declared_count_by_the_bytes_remaining_in_the_chunk() -> XrfResult {
    let chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7])?;

    assert_eq!(chunk.new_bounded_vec::<u64>(1, 8, "test records")?.capacity(), 1);

    let error: String = chunk
      .new_bounded_vec::<u64>(2, 8, "test records")
      .expect_err("expect the declared count to exceed the remaining bytes")
      .to_string();

    assert!(
      error.contains("test records declares 2 entries"),
      "Unexpected error: {error}"
    );
    assert!(error.contains("only 8 remain"), "Unexpected error: {error}");

    Ok(())
  }

  /// A four byte record, the smallest a test needs.
  #[derive(Debug, PartialEq)]
  struct Record(u32);

  impl ChunkReadWrite for Record {
    fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
      Ok(Self(reader.read_u32::<T>()?))
    }

    fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
      Ok(writer.write_u32::<T>(self.0)?)
    }
  }

  #[test]
  fn reads_a_packed_run_of_records_counted_by_its_size() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[1, 0, 0, 0, 2, 0, 0, 0])?;

    assert_eq!(
      chunk.read_xr_records::<XRayByteOrder, Record>(4, "test records")?,
      vec![Record(1), Record(2)]
    );
    assert!(chunk.is_ended());

    Ok(())
  }

  #[test]
  fn refuses_a_record_run_that_does_not_divide_into_whole_records() -> XrfResult {
    let error: String = ChunkReader::from_bytes(&[1, 0, 0, 0, 2])?
      .read_xr_records::<XRayByteOrder, Record>(4, "test records")
      .expect_err("expect the trailing byte to be refused")
      .to_string();

    assert_eq!(
      error,
      "Invalid error: Unexpected test records of 5 bytes, which does not divide into 4 byte records"
    );

    Ok(())
  }

  #[test]
  fn rejects_u16_vector_count_larger_than_remaining_payload() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[2, 0, 0, 0, 1, 0])?;
    let error: String = chunk
      .read_u16_vector::<XRayByteOrder>()
      .expect_err("expect the declared count to exceed the payload")
      .to_string();

    assert!(
      error.contains("u16 vector declares 2 entries"),
      "Unexpected error: {error}"
    );
    assert!(error.contains("only 2 remain"), "Unexpected error: {error}");

    Ok(())
  }
}
