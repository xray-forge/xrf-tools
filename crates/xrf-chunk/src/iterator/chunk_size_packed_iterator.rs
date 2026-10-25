use std::io::SeekFrom;

use byteorder::ReadBytesExt;
use fileslice::FileSlice;
use xrf_error::{XrfError, XrfResult};

use crate::chunk_constants::CHUNK_SIZE_FIELD_SIZE;
use crate::{ChunkDataSource, ChunkReader, XRayByteOrder};

/// Walks a payload stored as `[size][content]` records, where each size counts its own four bytes.
///
/// Hands each record over as a reader whose id is its ordinal and whose window holds the content alone. Advances the
/// parent reader's cursor past every record it hands over, and stops at the first malformed size, handing over its
/// error once.
pub struct ChunkSizePackedIterator<'a, T: ChunkDataSource = FileSlice> {
  pub index: u32,
  pub reader: &'a mut ChunkReader<T>,
  failed: bool,
}

impl<'a, T: ChunkDataSource> ChunkSizePackedIterator<'a, T> {
  /// Iterates from the start of the reader's payload.
  pub fn from_start(reader: &'a mut ChunkReader<T>) -> XrfResult<Self> {
    reader.reset_pos()?;

    Ok(Self::from_current(reader))
  }

  /// Iterates from wherever the reader's cursor already is, numbering records from zero there.
  pub fn from_current(reader: &'a mut ChunkReader<T>) -> Self {
    Self {
      index: 0,
      reader,
      failed: false,
    }
  }

  fn fail(&mut self, error: XrfError) -> Option<XrfResult<ChunkReader<T>>> {
    self.failed = true;

    Some(Err(error))
  }
}

impl<T: ChunkDataSource> Iterator for ChunkSizePackedIterator<'_, T> {
  type Item = XrfResult<ChunkReader<T>>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.failed || self.reader.is_ended() {
      return None;
    }

    let position: u64 = match self.reader.data.get_seek() {
      Ok(position) => position,
      Err(error) => return self.fail(error.into()),
    };

    let remaining: u64 = self.reader.read_bytes_remain();

    if remaining < CHUNK_SIZE_FIELD_SIZE {
      return self.fail(XrfError::new_invalid_error(format!(
        "Incomplete packed chunk size at position {position}, expected {CHUNK_SIZE_FIELD_SIZE} bytes but only \
         {remaining} remain"
      )));
    }

    // todo: Hardcoded byte order, should be part of generics.
    let size: u64 = match self.reader.read_u32::<XRayByteOrder>() {
      Ok(size) => u64::from(size),
      Err(error) => return self.fail(error.into()),
    };

    let id: u32 = self.index;

    if size < CHUNK_SIZE_FIELD_SIZE {
      return self.fail(XrfError::new_invalid_error(format!(
        "Packed chunk {id} at position {position} declares invalid size {size}; size includes the \
         {CHUNK_SIZE_FIELD_SIZE}-byte header"
      )));
    }

    let end_position: u64 = match position.checked_add(size) {
      Some(end_position) => end_position,
      None => {
        return self.fail(XrfError::new_invalid_error(format!(
          "Packed chunk {id} size {size} overflows its position {position}"
        )));
      }
    };

    if end_position > self.reader.end_pos() {
      return self.fail(XrfError::new_invalid_error(format!(
        "Packed chunk {id} at position {position} declares {size} bytes, beyond source end {}",
        self.reader.end_pos()
      )));
    }

    self.index += 1;

    // The size field is already read, so the content is what is left of the declared size.
    if let Err(error) = self
      .reader
      .data
      .set_seek(SeekFrom::Current((size - CHUNK_SIZE_FIELD_SIZE) as i64))
    {
      return self.fail(error.into());
    }

    Some(Ok(ChunkReader {
      id,
      size,
      position,
      data: self.reader.data.slice(position + CHUNK_SIZE_FIELD_SIZE..end_position),
    }))
  }
}

#[cfg(test)]
mod tests {
  use std::io::SeekFrom;

  use xrf_error::XrfResult;

  use crate::{ChunkDataSource, ChunkReader, ChunkSizePackedIterator, InMemoryChunkDataSource};

  #[test]
  fn test_iterate_empty() -> XrfResult {
    // A chunk declaring no payload, since a reader cannot be opened over an empty source.
    let mut chunk_reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_bytes(&[0; 8])?.read_child_by_index(0)?;

    if ChunkSizePackedIterator::from_start(&mut chunk_reader)?.next().is_some() {
      panic!("No iterations expected in empty data");
    }

    if ChunkSizePackedIterator::from_current(&mut chunk_reader)
      .next()
      .is_some()
    {
      panic!("No iterations expected in empty data");
    }

    Ok(())
  }

  #[test]
  fn test_iterate_single() -> XrfResult {
    let mut chunk_reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_source(InMemoryChunkDataSource::from_buffer(&[5, 0, 0, 0, 255]))?;

    let mut vec: Vec<ChunkReader<InMemoryChunkDataSource>> = Vec::new();

    for it in ChunkSizePackedIterator::from_start(&mut chunk_reader)? {
      vec.push(it?);
    }

    assert_eq!(vec.len(), 1, "Expected count to be 1");
    assert_eq!(vec[0].id, 0, "Expected id to be 0");
    assert_eq!(vec[0].size, 5, "Expected size to be 5");
    assert_eq!(vec[0].position, 0, "Expected position to be 0");

    Ok(())
  }

  #[test]
  fn test_iterate_few_start() -> XrfResult {
    let mut chunk_reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_source(InMemoryChunkDataSource::from_buffer(&[
        8, 0, 0, 0, 255, 255, 255, 255, 12, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255,
      ]))?;

    let mut vec: Vec<ChunkReader<InMemoryChunkDataSource>> = Vec::new();

    for it in ChunkSizePackedIterator::from_start(&mut chunk_reader)? {
      vec.push(it?);
    }

    assert_eq!(vec.len(), 2, "Expected count to be 2");
    assert_eq!(vec[0].id, 0, "Expected [0] id to be 0");
    assert_eq!(vec[0].size, 8, "Expected [0] size to be 8");
    assert_eq!(vec[0].position, 0, "Expected [0] position to be 0");
    assert_eq!(vec[1].id, 1, "Expected [1] id to be 1");
    assert_eq!(vec[1].size, 12, "Expected [1] size to be 12");
    assert_eq!(vec[1].position, 8, "Expected [1] position to be 8");

    Ok(())
  }

  #[test]
  fn test_iterate_few_mid() -> XrfResult {
    let mut chunk_reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_source(InMemoryChunkDataSource::from_buffer(&[
        8, 0, 0, 0, 255, 255, 255, 255, 12, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255,
      ]))?;

    chunk_reader.data.set_seek(SeekFrom::Start(8))?;

    let mut vec: Vec<ChunkReader<InMemoryChunkDataSource>> = Vec::new();

    for it in ChunkSizePackedIterator::from_current(&mut chunk_reader) {
      vec.push(it?);
    }

    // Records are numbered from where the walk starts, so the second record of the payload is this walk's first.
    assert_eq!(vec.len(), 1, "Expected count to be 1");
    assert_eq!(vec[0].id, 0, "Expected [0] id to be 0");
    assert_eq!(vec[0].size, 12, "Expected [0] size to be 12");
    assert_eq!(vec[0].position, 8, "Expected [0] position to be 8");

    Ok(())
  }

  #[test]
  fn rejects_size_smaller_than_header() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_source(InMemoryChunkDataSource::from_buffer(&[3, 0, 0, 0]))?;
    let error: String = ChunkSizePackedIterator::from_start(&mut reader)?
      .next()
      .expect("Expected one invalid packed chunk")
      .expect_err("Expected undersized packed chunk to fail")
      .to_string();

    assert!(error.contains("declares invalid size 3"), "Unexpected error: {error}");

    Ok(())
  }

  #[test]
  fn rejects_packed_chunk_data_beyond_source_end() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_source(InMemoryChunkDataSource::from_buffer(&[8, 0, 0, 0, 0]))?;
    let error: String = ChunkSizePackedIterator::from_start(&mut reader)?
      .next()
      .expect("Expected one invalid packed chunk")
      .expect_err("Expected oversized packed chunk to fail")
      .to_string();

    assert!(error.contains("beyond source end"), "Unexpected error: {error}");

    Ok(())
  }
}
