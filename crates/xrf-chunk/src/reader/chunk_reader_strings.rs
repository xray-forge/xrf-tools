use std::io::{Read, SeekFrom};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::encode_w1251_bytes_to_string;

use crate::reader::chunk_line::{ChunkLine, is_line_terminator};
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

  /// Read one line terminated windows encoded string from file bytes, with the terminator that closed it.
  pub fn read_w1251_line(&mut self) -> XrfResult<ChunkLine> {
    self.read_w1251_line_limited(10_240)
  }

  /// Read one line terminated windows encoded string from file bytes, with a size limit on the value.
  ///
  /// Ends the value at the first `\r` or `\n` and then consumes the whole run of them, which is what
  /// `advance_term_string` does (`xray-16/src/xrCore/FS.cpp:384`) and so what every field the engine reads with
  /// `IReader::r_string` allows: `\r\n`, a bare `\n`, a bare `\r` and any repetition all end one string. The run is
  /// reported rather than dropped, so a writer can put back what the file carried.
  ///
  /// The engine advances before it tests, so a value opening on a terminator comes back one byte long there rather
  /// than empty. That off-by-one is not reproduced; either reading re-emits the same bytes.
  ///
  /// # Errors
  ///
  /// Returns an error when the data ends before any terminator, where the engine takes what is left. A field read
  /// this way is followed by more of its record, so reaching the end means the value has eaten it.
  pub fn read_w1251_line_limited(&mut self, limit: usize) -> XrfResult<ChunkLine> {
    let mut buffer: [u8; STRING_READ_BUFFER_SIZE] = [0u8; STRING_READ_BUFFER_SIZE];
    let mut collected: Vec<u8> = Vec::new();
    let mut terminator_start: Option<usize> = None;

    loop {
      let bytes_read: usize = self.read(&mut buffer)?;

      if bytes_read == 0 {
        // The run reaches the end of the data, so no byte is left after it to stop the skip at.
        let Some(start) = terminator_start else {
          return Err(XrfError::new_no_terminator_error(
            "Line terminator is not found in buffer, no data to be read",
          ));
        };

        return self.take_line(&collected, start, collected.len());
      }

      let search_start: usize = collected.len();

      collected.extend_from_slice(&buffer[..bytes_read]);

      if terminator_start.is_none() {
        terminator_start = collected[search_start..]
          .iter()
          .position(|it| is_line_terminator(*it))
          .map(|it| search_start + it);
      }

      let Some(start) = terminator_start else {
        if collected.len() > limit {
          return Err(XrfError::new_parsing_error(
            "Cannot parse string, reading data over buffer size limit",
          ));
        }

        continue;
      };

      if start > limit {
        return Err(XrfError::new_parsing_error(
          "Cannot parse string, reading data over buffer size limit",
        ));
      }

      // Until a byte past the run is read, the run may go on into what has not been read yet.
      if let Some(offset) = collected[start..].iter().position(|it| !is_line_terminator(*it)) {
        return self.take_line(&collected, start, start + offset);
      }
    }
  }

  /// Splits a read-ahead buffer into the value and its terminator run, giving back the bytes past the run.
  fn take_line(&mut self, collected: &[u8], start: usize, end: usize) -> XrfResult<ChunkLine> {
    let extra_bytes: i64 = i64::try_from(collected.len() - end)
      .map_err(|_| XrfError::new_invalid_error("String read-ahead exceeds the supported seek range"))?;

    self.data.set_seek(SeekFrom::Current(-extra_bytes))?;

    Ok(ChunkLine {
      value: encode_w1251_bytes_to_string(&collected[..start])?,
      terminator: encode_w1251_bytes_to_string(&collected[start..end])?,
    })
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
  use crate::reader::chunk_line::ChunkLine;
  use crate::reader::chunk_reader::ChunkReader;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;

  fn line(value: &str, terminator: &str) -> ChunkLine {
    ChunkLine {
      value: String::from(value),
      terminator: String::from(terminator),
    }
  }

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
  fn test_read_w1251_line_empty() -> XrfResult {
    // A chunk declaring no payload, since a reader cannot be opened over an empty source.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0; 8])?.read_child_by_index(0)?;

    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(
      chunk.read_w1251_line().unwrap_err().to_string(),
      "Missing terminator error: Line terminator is not found in buffer, no data to be read",
      "Expect error on empty read"
    );
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    Ok(())
  }

  #[test]
  fn refuses_a_line_the_data_ends_before_terminating() -> XrfResult {
    // The engine would take what is left, but a field read this way is followed by more of its record: running to the
    // end means the value has swallowed whatever came after it.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"toggle")?;

    assert_eq!(
      chunk.read_w1251_line().unwrap_err().to_string(),
      "Missing terminator error: Line terminator is not found in buffer, no data to be read"
    );

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
  fn test_read_w1251_line_empty_remaining_data() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[b'\r', b'\n', 0, 0, 0])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 5, "Expect 5 bytes remaining");

    assert_eq!(chunk.read_w1251_line()?, line("", "\r\n"), "Expect empty string");
    assert_eq!(chunk.cursor_pos(), 2, "Expect 2 byte read");
    assert_eq!(chunk.read_bytes_remain(), 3, "Expect 3 bytes remaining");

    Ok(())
  }

  #[test]
  fn test_read_w1251_line_few() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"abc\r\ncba\r\n")?;

    assert_eq!(chunk.read_bytes_remain(), 10, "Expect 10 bytes remaining");
    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");

    assert_eq!(chunk.read_w1251_line()?, line("abc", "\r\n"), "Expect string read");
    assert_eq!(chunk.cursor_pos(), 5, "Expect 5 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 5, "Expect 5 bytes remaining");

    assert_eq!(chunk.read_w1251_line()?, line("cba", "\r\n"), "Expect string read");
    assert_eq!(chunk.cursor_pos(), 10, "Expect 10 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 0, "Expect 0 bytes remaining");

    Ok(())
  }

  #[test]
  fn reads_a_line_terminated_by_a_bare_newline() -> XrfResult {
    // What three Gunslinger animation banks carry, and what `is_term` accepts: one `\n` ends the name, and the count
    // that follows it is left where the record expects it.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"toggle\n\x01\x00\x00\x00")?;

    assert_eq!(chunk.read_w1251_line()?, line("toggle", "\n"));
    assert_eq!(chunk.cursor_pos(), 7);
    assert_eq!(chunk.read_remaining()?, [1, 0, 0, 0]);

    Ok(())
  }

  #[test]
  fn reads_a_line_terminated_by_a_bare_carriage_return() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"toggle\rnext")?;

    assert_eq!(chunk.read_w1251_line()?, line("toggle", "\r"));
    assert_eq!(chunk.read_remaining()?, b"next");

    Ok(())
  }

  #[test]
  fn takes_a_whole_run_of_terminators_as_one() -> XrfResult {
    // `advance_term_string` skips every terminator it finds rather than a fixed pair, so a blank line between entries
    // belongs to the one before it rather than opening an empty one.
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"abc\r\n\r\n\ncba\r\n")?;

    assert_eq!(chunk.read_w1251_line()?, line("abc", "\r\n\r\n\n"));
    assert_eq!(chunk.read_w1251_line()?, line("cba", "\r\n"));
    assert_eq!(chunk.read_bytes_remain(), 0);

    Ok(())
  }

  #[test]
  fn takes_a_run_of_terminators_that_reaches_the_end_of_the_data() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"abc\n\n")?;

    assert_eq!(chunk.read_w1251_line()?, line("abc", "\n\n"));
    assert_eq!(chunk.read_bytes_remain(), 0);

    Ok(())
  }

  #[test]
  fn reads_a_line_terminator_split_across_internal_buffers() -> XrfResult {
    let mut bytes: Vec<u8> = vec![b'a'; STRING_READ_BUFFER_SIZE - 1];
    bytes.extend_from_slice(b"\r\nnext");
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;

    assert_eq!(
      chunk.read_w1251_line_limited(STRING_READ_BUFFER_SIZE - 1)?,
      line(&"a".repeat(STRING_READ_BUFFER_SIZE - 1), "\r\n")
    );
    assert_eq!(chunk.cursor_pos(), (STRING_READ_BUFFER_SIZE + 1) as u64);
    assert_eq!(chunk.read_remaining()?, b"next");

    Ok(())
  }

  #[test]
  fn reads_a_terminator_run_continuing_past_an_internal_buffer() -> XrfResult {
    // The run's end is what stops the read, so a run filling the rest of a buffer has to keep the reader going rather
    // than cut the line where the buffer happened to end.
    let mut bytes: Vec<u8> = vec![b'a'];
    bytes.extend_from_slice(&[b'\n'; STRING_READ_BUFFER_SIZE]);
    bytes.extend_from_slice(b"next");
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;

    assert_eq!(
      chunk.read_w1251_line()?,
      line("a", &"\n".repeat(STRING_READ_BUFFER_SIZE))
    );
    assert_eq!(chunk.read_remaining()?, b"next");

    Ok(())
  }

  #[test]
  fn test_read_w1251_line_limited_over_limit() -> XrfResult {
    let mut chunk: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[b'a'; 1024])?;

    assert_eq!(chunk.cursor_pos(), 0, "Expect 0 bytes read");
    assert_eq!(chunk.read_bytes_remain(), 1024, "Expect 1024 bytes remaining");

    assert_eq!(
      chunk.read_w1251_line_limited(500).unwrap_err().to_string(),
      "Parsing error: Cannot parse string, reading data over buffer size limit",
      "Expect buffer limit error"
    );

    Ok(())
  }
}
