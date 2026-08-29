use std::io::Read;

use byteorder::{ByteOrder, ReadBytesExt};
use xrf_error::{XrfError, XrfResult};

use crate::chunk_trait::ChunkReadWrite;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::{ChunkReadWriteList, ChunkReadWriteOptional, ChunkReader};

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
}

impl<D: ChunkDataSource> ChunkReader<D> {
  /// Create a vector for a record count this chunk declares, bounded by the bytes remaining in it.
  ///
  /// `min_record_size` counts only the bytes every record consumes unconditionally; see
  /// [`xrf_utils::assert_count_fits`].
  pub fn new_bounded_vec<C>(&self, count: u64, min_record_size: u64, what: &str) -> XrfResult<Vec<C>> {
    xrf_utils::new_bounded_vec(count, self.read_bytes_remain(), min_record_size, what)
  }

  /// Read serialized vector from chunk, where u32 count N is followed by N u16 entries.
  pub fn read_u16_vector<T: ByteOrder>(&mut self) -> XrfResult<Vec<u16>> {
    let count: u32 = self.read_u32::<T>()?;
    let available_count: u64 = self.read_bytes_remain() / size_of::<u16>() as u64;

    if u64::from(count) > available_count {
      return Err(XrfError::new_invalid_error(format!(
        "u16 vector declares {count} entries, but only {available_count} fit in the remaining payload"
      )));
    }

    let capacity: usize = usize::try_from(count)
      .map_err(|_| XrfError::new_invalid_error(format!("u16 vector count {count} exceeds the platform limit")))?;

    let mut vector: Vec<u16> = Vec::with_capacity(capacity);

    for _ in 0..count {
      vector.push(self.read_u16::<T>()?)
    }

    Ok(vector)
  }

  /// Read raw bytes.
  pub fn read_bytes(&mut self, count: usize) -> XrfResult<Vec<u8>> {
    let count: u64 = u64::try_from(count)
      .map_err(|_| XrfError::new_invalid_error("Requested byte count exceeds the supported source range"))?;

    if count > self.read_bytes_remain() {
      return Err(XrfError::new_invalid_error(format!(
        "Requested {count} bytes, but only {} remain in the chunk",
        self.read_bytes_remain()
      )));
    }

    let count: usize = usize::try_from(count)
      .map_err(|_| XrfError::new_invalid_error("Requested byte count exceeds the platform limit"))?;

    Ok(self.data.read_bytes(count)?)
  }

  /// Read all remaining raw bytes.
  pub fn read_remaining(&mut self) -> XrfResult<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();

    self.read_to_end(&mut buf)?;

    Ok(buf)
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::XRayByteOrder;
  use crate::reader::chunk_reader::ChunkReader;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;

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
    assert_eq!(ChunkReader::from_bytes(&[])?.read_remaining()?, Vec::<u8>::new());

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

  #[test]
  fn rejects_u16_vector_count_larger_than_remaining_payload() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[2, 0, 0, 0, 1, 0])?;
    let error: String = chunk
      .read_u16_vector::<XRayByteOrder>()
      .expect_err("expect the declared count to exceed the payload")
      .to_string();

    assert!(error.contains("declares 2 entries"), "Unexpected error: {error}");
    assert!(error.contains("only 1 fit"), "Unexpected error: {error}");

    Ok(())
  }
}
