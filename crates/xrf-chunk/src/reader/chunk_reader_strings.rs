use std::io::{Read, SeekFrom};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::encode_w1251_bytes_to_string;

use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;

const STRING_READ_BUFFER_SIZE: usize = 256;

impl<D: ChunkDataSource> ChunkReader<D> {
  /// Read null terminated windows encoded string from file bytes.
  pub fn read_w1251_string(&mut self) -> XrfResult<String> {
    self.read_w1251_string_limited(10_240)
  }

  /// Read null terminated windows encoded string from file bytes with size limit.
  pub fn read_w1251_string_limited(&mut self, limit: usize) -> XrfResult<String> {
    self.read_w1251_terminated_limited(
      b"\0",
      limit,
      "Null terminator is not found in buffer, no data to be read",
    )
  }

  /// Read \r\n terminated windows encoded string from file bytes.
  pub fn read_w1251_rn_string(&mut self) -> XrfResult<String> {
    self.read_w1251_rn_string_limited(10_240)
  }

  /// Read \r\n terminated windows encoded string from file bytes.
  pub fn read_w1251_rn_string_limited(&mut self, limit: usize) -> XrfResult<String> {
    self.read_w1251_terminated_limited(b"\r\n", limit, "RN sequence is not found in buffer, no data to be read")
  }

  fn read_w1251_terminated_limited(
    &mut self,
    terminator: &[u8],
    limit: usize,
    missing_terminator_message: &'static str,
  ) -> XrfResult<String> {
    let mut buffer: [u8; STRING_READ_BUFFER_SIZE] = [0u8; STRING_READ_BUFFER_SIZE];
    let mut collected: Vec<u8> = Vec::new();
    let maximum_unterminated_size: usize = limit.saturating_add(terminator.len().saturating_sub(1));

    loop {
      let bytes_read: usize = self.read(&mut buffer)?;

      if bytes_read == 0 {
        return Err(XrfError::new_no_terminator_error(missing_terminator_message));
      }

      let previous_size: usize = collected.len();
      collected.extend_from_slice(&buffer[..bytes_read]);
      let search_start: usize = previous_size.saturating_sub(terminator.len().saturating_sub(1));

      if let Some(relative_position) = collected[search_start..]
        .windows(terminator.len())
        .position(|candidate| candidate == terminator)
      {
        let position: usize = search_start + relative_position;

        if position > limit {
          return Err(XrfError::new_parsing_error(
            "Cannot parse string, reading data over buffer size limit",
          ));
        }

        let consumed_size: usize = position + terminator.len();
        let extra_bytes: usize = collected.len() - consumed_size;
        let extra_bytes: i64 = i64::try_from(extra_bytes)
          .map_err(|_| XrfError::new_invalid_error("String read-ahead exceeds the supported seek range"))?;

        self.data.set_seek(SeekFrom::Current(-extra_bytes))?;
        collected.truncate(position);

        break;
      }

      if collected.len() > maximum_unterminated_size {
        return Err(XrfError::new_parsing_error(
          "Cannot parse string, reading data over buffer size limit",
        ));
      }
    }

    Ok(encode_w1251_bytes_to_string(&collected)?)
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use super::STRING_READ_BUFFER_SIZE;
  use crate::reader::chunk_reader::ChunkReader;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;

  #[test]
  fn test_read_w1251_string_empty() -> XrfResult {
    // A chunk declaring no payload, since a reader cannot be opened over an empty source.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0; 8])?.read_child_by_index(0)?;

    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(
      chunk.read_w1251_string().unwrap_err().to_string(),
      "Missing terminator error: Null terminator is not found in buffer, no data to be read",
      "Expect error on empty read"
    );
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    Ok(())
  }

  #[test]
  fn test_read_w1251_string() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 1, "Expect 1 byte remaining");

    assert_eq!(chunk.read_w1251_string()?, "", "Expect empty string with terminator");
    assert_eq!(chunk.cursor_pos(), 1, "Expect 1 byte read");
    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");

    Ok(())
  }

  #[test]
  fn test_read_w1251_string_empty_remaining_data() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0, 0, 0, 0])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 4, "Expect 4 bytes remaining");

    assert_eq!(chunk.read_w1251_string()?, "", "Expect empty string");
    assert_eq!(chunk.cursor_pos(), 1, "Expect 1 byte read");
    assert_eq!(chunk.read_bytes_remain(), 3, "Expect 3 bytes remaining");

    Ok(())
  }

  #[test]
  fn test_read_w1251_string_strings_few() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_bytes(&[b'a', b'b', b'c', 0, b'c', b'b', b'a', 0])?;

    assert_eq!(chunk.read_bytes_remain(), 8, "Expect 8 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(chunk.read_w1251_string()?, "abc", "Expect string read");
    assert_eq!(chunk.cursor_pos(), 4, "Expect 4 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 4, "Expect 4 bytes remaining");

    assert_eq!(chunk.read_w1251_string()?, "cba", "Expect string read");
    assert_eq!(chunk.cursor_pos(), 8, "Expect 8 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");

    Ok(())
  }

  #[test]
  fn test_read_w1251_string_limited_over_limit() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[b'a'; 1024])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 1024, "Expect 1024 bytes remaining");

    assert_eq!(
      chunk.read_w1251_string_limited(500).unwrap_err().to_string(),
      "Parsing error: Cannot parse string, reading data over buffer size limit",
      "Expect buffer limit error"
    );

    Ok(())
  }

  #[test]
  fn accepts_null_terminated_content_at_the_exact_limit_despite_read_ahead() -> XrfResult {
    let mut bytes: Vec<u8> = vec![b'a', 0];
    bytes.extend_from_slice(&[b'x'; 254]);
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;

    assert_eq!(chunk.read_w1251_string_limited(1)?, "a");
    assert_eq!(chunk.cursor_pos(), 2);
    assert_eq!(chunk.read_bytes_remain(), 254);

    Ok(())
  }

  #[test]
  fn test_read_w1251_rn_string_empty() -> XrfResult {
    // A chunk declaring no payload, since a reader cannot be opened over an empty source.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0; 8])?.read_child_by_index(0)?;

    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(
      chunk.read_w1251_rn_string().unwrap_err().to_string(),
      "Missing terminator error: RN sequence is not found in buffer, no data to be read",
      "Expect error on empty read"
    );
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    Ok(())
  }

  #[test]
  fn test_read_w1251_string_empty_null() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 1, "Expect 1 byte remaining");

    assert_eq!(chunk.read_w1251_string()?, "", "Expect empty string with terminator");
    assert_eq!(chunk.cursor_pos(), 1, "Expect 1 byte read");
    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");

    Ok(())
  }

  #[test]
  fn test_read_w1251_rn_string_empty_remaining_data() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[b'\r', b'\n', 0, 0, 0])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 5, "Expect 5 bytes remaining");

    assert_eq!(chunk.read_w1251_rn_string()?, "", "Expect empty string");
    assert_eq!(chunk.cursor_pos(), 2, "Expect 2 byte read");
    assert_eq!(chunk.read_bytes_remain(), 3, "Expect 3 bytes remaining");

    Ok(())
  }

  #[test]
  fn test_read_w1251_rn_string_few() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"abc\r\ncba\r\n")?;

    assert_eq!(chunk.read_bytes_remain(), 10, "Expect 10 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(chunk.read_w1251_rn_string()?, "abc", "Expect string read");
    assert_eq!(chunk.cursor_pos(), 5, "Expect 5 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 5, "Expect 5 bytes remaining");

    assert_eq!(chunk.read_w1251_rn_string()?, "cba", "Expect string read");
    assert_eq!(chunk.cursor_pos(), 10, "Expect 10 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");

    Ok(())
  }

  #[test]
  fn reads_rn_terminator_split_across_internal_buffers() -> XrfResult {
    let mut bytes: Vec<u8> = vec![b'a'; STRING_READ_BUFFER_SIZE - 1];
    bytes.extend_from_slice(b"\r\nnext");
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;

    assert_eq!(
      chunk.read_w1251_rn_string_limited(STRING_READ_BUFFER_SIZE - 1)?,
      "a".repeat(STRING_READ_BUFFER_SIZE - 1)
    );
    assert_eq!(chunk.cursor_pos(), (STRING_READ_BUFFER_SIZE + 1) as u64);
    assert_eq!(chunk.read_remaining()?, b"next");

    Ok(())
  }

  #[test]
  fn test_read_w1251_rn_string_limited_over_limit() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[b'a'; 1024])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 1024, "Expect 1024 bytes remaining");

    assert_eq!(
      chunk.read_w1251_rn_string_limited(500).unwrap_err().to_string(),
      "Parsing error: Cannot parse string, reading data over buffer size limit",
      "Expect buffer limit error"
    );

    Ok(())
  }
}
