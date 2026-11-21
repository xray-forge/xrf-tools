use xrf_error::{XrfError, XrfResult};

use crate::read_write::chunk_record::ChunkRecord;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;

/// Arrays of fixed-size records, decoded in one pass over their bytes where the source lends them.
impl<D: ChunkDataSource> ChunkReader<D> {
  /// Read `count` records stored back to back from the cursor.
  ///
  /// `what` names the array in the errors.
  ///
  /// # Errors
  ///
  /// Returns an error, before reading anything, when fewer than `count` whole records remain in the chunk.
  pub fn read_records<R: ChunkRecord<SIZE>, const SIZE: usize>(&mut self, count: u64, what: &str) -> XrfResult<Vec<R>> {
    xrf_utils::assert_count_fits(count, self.read_bytes_remain(), SIZE as u64, what)?;

    // The bound above keeps the product within the chunk, so it fits.
    let size: usize = usize::try_from(count * SIZE as u64)
      .map_err(|_| XrfError::new_invalid_error(format!("{what} of {count} records is over the platform limit")))?;

    self.read_with(size, |bytes| {
      bytes.as_chunks::<SIZE>().0.iter().map(R::decode).collect()
    })
  }

  /// Read the rest of the chunk as records stored back to back, which is how the engine reads a chunk it casts onto an
  /// array: no count is stored, the size alone says how many there are.
  ///
  /// # Errors
  ///
  /// Returns an error when the rest of the chunk is not a whole number of records.
  pub fn read_remaining_records<R: ChunkRecord<SIZE>, const SIZE: usize>(&mut self, what: &str) -> XrfResult<Vec<R>> {
    let size: u64 = self.read_bytes_remain();

    if !size.is_multiple_of(SIZE as u64) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected {what} of {size} bytes, which does not divide into {SIZE} byte records"
      )));
    }

    self.read_records(size / SIZE as u64, what)
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::read_write::chunk_record::ChunkRecord;
  use crate::reader::chunk_reader::ChunkReader;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;
  use crate::writer::chunk_writer::ChunkWriter;

  /// A record whose two fields have different widths, so a decoder reading them in the wrong order shows.
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  struct Pair {
    id: u16,
    value: u32,
  }

  impl ChunkRecord<6> for Pair {
    fn decode(bytes: &[u8; 6]) -> Self {
      Self {
        id: u16::from_le_bytes([bytes[0], bytes[1]]),
        value: u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
      }
    }

    fn encode(&self) -> [u8; 6] {
      let [a, b] = self.id.to_le_bytes();
      let [c, d, e, f] = self.value.to_le_bytes();

      [a, b, c, d, e, f]
    }
  }

  fn pairs() -> Vec<Pair> {
    vec![
      Pair {
        id: 1,
        value: 0x0302_0100,
      },
      Pair { id: 0xFFFF, value: 7 },
    ]
  }

  #[test]
  fn reads_back_what_write_records_wrote_and_moves_past_it() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_records(&pairs())?;
    writer.buffer.push(0xEE);

    assert_eq!(&writer.buffer[..6], [1, 0, 0, 1, 2, 3]);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(writer.buffer)?;
    let read: Vec<Pair> = reader.read_records(2, "pairs")?;

    assert_eq!(read, pairs());
    assert_eq!(
      reader.read_bytes_remain(),
      1,
      "the cursor to stop right after the records"
    );

    Ok(())
  }

  #[test]
  fn refuses_a_count_past_the_chunk_before_reading() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0; 11])?;
    let error: String = reader
      .read_records::<Pair, 6>(2, "pairs")
      .expect_err("expect two six-byte records not to fit eleven bytes")
      .to_string();

    assert!(error.contains("pairs declares 2 entries"), "Unexpected error: {error}");
    assert_eq!(
      reader.cursor_pos(),
      0,
      "a refused count to leave the cursor where it was"
    );

    Ok(())
  }

  #[test]
  fn reads_the_rest_as_whole_records_only() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_records(&pairs())?;

    let read: Vec<Pair> = ChunkReader::from_vec(writer.buffer.clone())?.read_remaining_records("pairs")?;

    assert_eq!(read, pairs());

    writer.buffer.push(0);

    let error: String = ChunkReader::from_vec(writer.buffer)?
      .read_remaining_records::<Pair, 6>("pairs")
      .expect_err("expect a trailing byte to be refused")
      .to_string();

    assert_eq!(
      error,
      "Invalid error: Unexpected pairs of 13 bytes, which does not divide into 6 byte records"
    );

    Ok(())
  }
}
