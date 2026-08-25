use std::fmt;
use std::fs::File;
use std::io::{Read, SeekFrom};

use fileslice::FileSlice;
use xrf_error::{XrfError, XrfResult};

use crate::iterator::chunk_iterator::ChunkIterator;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::source::chunk_memory_source::InMemoryChunkDataSource;

#[derive(Clone)]
pub struct ChunkReader<T: ChunkDataSource = FileSlice> {
  pub id: u32,
  pub size: u64,
  pub position: u64,
  pub data: T,
}

impl ChunkReader<FileSlice> {
  /// Creates a reader over the supplied file-slice boundaries.
  ///
  /// # Errors
  ///
  /// Returns an error when the slice is empty.
  pub fn from_slice(slice: FileSlice) -> XrfResult<Self> {
    if slice.is_empty() {
      return Err(XrfError::new_invalid_error("Failed to create chunk from empty source"));
    }

    Ok(Self {
      id: 0,
      size: slice.len() as u64,
      position: slice.start_pos(),
      data: slice,
    })
  }
}

impl ChunkReader<InMemoryChunkDataSource> {
  /// Creates a reader spanning the whole file, held in memory.
  ///
  /// Reading a chunked format is per field — a `u32` here, an `f32` there — and a file-backed source answers each one
  /// with its own positioned read: 296k of them, 167ms, to read the 1.2MB particles library that costs two reads to
  /// hold. The bytes are taken through a [`FileSlice`] rather than off the handle so the file's own cursor is left where
  /// the caller had it, and reading still starts at offset 0. Use [`Self::from_slice`] for the windowed form.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is empty.
  pub fn from_file(file: File) -> XrfResult<Self> {
    let mut slice: FileSlice = FileSlice::new(file);
    let mut buffer: Vec<u8> = Vec::with_capacity(slice.len());

    slice.read_to_end(&mut buffer)?;

    if buffer.is_empty() {
      return Err(XrfError::new_invalid_error("Failed to create chunk from empty source"));
    }

    Self::from_vec(buffer)
  }

  /// Creates a reader over a copied in-memory byte buffer.
  ///
  /// Prefer [`Self::from_vec`] when the bytes are already owned — this copies them.
  pub fn from_bytes(buf: &[u8]) -> XrfResult<Self> {
    Self::from_source(InMemoryChunkDataSource::from_buffer(buf))
  }

  /// Creates a reader over owned bytes without copying them.
  pub fn from_vec(buf: Vec<u8>) -> XrfResult<Self> {
    Self::from_source(InMemoryChunkDataSource::from_vec(buf))
  }

  /// Creates a reader over an in-memory chunk data source.
  pub fn from_source(source: InMemoryChunkDataSource) -> XrfResult<Self> {
    Ok(Self {
      id: 0,
      size: source.len(),
      position: 0,
      data: source,
    })
  }
}

impl<T: ChunkDataSource> ChunkReader<T> {
  /// Returns the absolute cursor position within the underlying source.
  pub fn cursor_pos(&self) -> u64 {
    self.data.cursor_pos()
  }

  /// Returns the absolute end position of the underlying source.
  pub fn end_pos(&self) -> u64 {
    self.data.end_pos()
  }

  /// Returns whether the cursor has reached the end of this chunk.
  pub fn is_ended(&self) -> bool {
    self.data.cursor_pos() == self.data.end_pos()
  }

  /// Returns whether at least one byte remains before the end of this chunk.
  pub fn has_data(&self) -> bool {
    self.data.cursor_pos() < self.data.end_pos()
  }

  /// Returns the number of bytes consumed from this chunk.
  pub fn read_bytes_len(&self) -> u64 {
    self.data.cursor_pos().saturating_sub(self.data.start_pos())
  }

  /// Returns the number of bytes remaining from the current cursor.
  pub fn read_bytes_remain(&self) -> u64 {
    self.data.end_pos().saturating_sub(self.data.cursor_pos())
  }

  /// Resets the cursor to the beginning of this chunk.
  pub fn reset_pos(&mut self) -> XrfResult<u64> {
    Ok(self.data.set_seek(SeekFrom::Start(0))?)
  }

  /// Reads the zero-based child at `id` after resetting and scanning this reader.
  ///
  /// The parent cursor is left after the last scanned child; the returned child owns a slice of
  /// the original source.
  ///
  /// # Errors
  ///
  /// Returns an error when child iteration fails or no child has that index.
  pub fn read_child_by_index(&mut self, id: u32) -> XrfResult<Self> {
    for (iteration, chunk) in ChunkIterator::from_start(self)?.enumerate() {
      let chunk: ChunkReader<T> = chunk?;

      if id as usize == iteration {
        return Ok(chunk);
      }
    }

    Err(XrfError::new_invalid_error(format!(
      "Attempt to read not existing chunk with id {} in chunk {}",
      id, self.id
    )))
  }

  /// Returns all children without changing this reader's cursor.
  pub fn get_children_cloned(&self) -> XrfResult<Vec<Self>> {
    ChunkIterator::from_start(&mut self.clone())?.collect()
  }

  /// Returns all children and advances this reader through the child sequence.
  pub fn read_children(&mut self) -> XrfResult<Vec<Self>> {
    ChunkIterator::<T>::from_start(self)?.collect()
  }

  /// Verifies that the cursor consumed the entire chunk.
  ///
  /// # Errors
  ///
  /// Returns a chunk-not-ended error containing the number of unread bytes.
  #[inline]
  pub fn assert_read(&self, message: &str) -> XrfResult {
    if self.is_ended() {
      Ok(())
    } else {
      Err(XrfError::new_chunk_not_ended_error(message, self.read_bytes_remain()))
    }
  }
}

impl fmt::Debug for ChunkReader {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      formatter,
      "Chunk {{ index: {}, size: {}, position: {} }}",
      self.id, self.size, self.position
    )
  }
}

#[cfg(test)]
mod tests {
  use fileslice::FileSlice;
  use xrf_error::XrfResult;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice, write_generated_test_resource,
  };

  use crate::reader::chunk_reader::ChunkReader;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;

  /// Lay out one chunk the way the format does: id, payload length, then the payload.
  fn new_chunk_bytes(id: u32, payload: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(8 + payload.len());

    bytes.extend_from_slice(&id.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);

    bytes
  }

  #[test]
  fn rejects_an_empty_source() -> XrfResult {
    // Only `from_slice` and `from_file` refuse an empty source - `from_bytes` hands back a zero-size reader - so this
    // case is the guard's sole coverage and has to keep a real file behind it.
    let path: String = build_relative_test_sample_file_path(file!(), "rejects_an_empty_source");

    write_generated_test_resource(&path, b"")?;

    let file: FileSlice = open_generated_test_resource_as_slice(&path)?;

    assert_eq!(file.start_pos(), 0);
    assert_eq!(file.end_pos(), 0);

    let result: XrfResult<ChunkReader> = ChunkReader::from_slice(file);

    assert!(result.is_err(), "File should be empty and fail to read data");
    assert_eq!(
      result.unwrap_err().to_string(),
      "Invalid error: Failed to create chunk from empty source",
      "Expect input error"
    );

    Ok(())
  }

  #[test]
  fn reads_a_single_empty_child() -> XrfResult {
    // Kept file-backed so reading through a `FileSlice`, not just constructing over one, stays covered here.
    let path: String = build_relative_test_sample_file_path(file!(), "reads_a_single_empty_child.chunk");

    write_generated_test_resource(&path, new_chunk_bytes(0, &[]))?;

    let file: FileSlice = open_generated_test_resource_as_slice(&path)?;

    assert_eq!(file.start_pos(), 0);
    assert_eq!(file.end_pos(), 8);

    let reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert!(reader.is_ended(), "Expect empty chunk");

    Ok(())
  }

  #[test]
  fn reads_five_empty_children() -> XrfResult {
    let bytes: Vec<u8> = (0..5).map(|id| new_chunk_bytes(id, &[])).collect::<Vec<_>>().concat();
    let children: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_bytes(&bytes)?.get_children_cloned()?;

    assert_eq!(children.len(), 5, "Expect five chunks");
    assert!(children.iter().all(|child| child.size == 0));

    Ok(())
  }

  #[test]
  fn reads_children_in_declaration_order() -> XrfResult {
    // Ids descend, so a reader that sorted them or assumed they ascend would report a different order.
    let bytes: Vec<u8> = [4, 3, 2, 1, 0].map(|id| new_chunk_bytes(id, &[])).concat();
    let children: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_bytes(&bytes)?.get_children_cloned()?;

    assert_eq!(children.len(), 5, "Expect five chunks");
    assert_eq!(
      children.iter().map(|child| child.id).collect::<Vec<u32>>(),
      vec![4, 3, 2, 1, 0]
    );
    assert!(children.iter().all(|child| child.size == 0));

    Ok(())
  }

  #[test]
  fn reads_children_with_payloads() -> XrfResult {
    let single: Vec<u8> = new_chunk_bytes(0, &[0xFF; 8]);
    let children: Vec<ChunkReader<InMemoryChunkDataSource>> =
      ChunkReader::from_bytes(&single)?.get_children_cloned()?;

    assert_eq!(children.len(), 1, "Expect single chunk");
    assert_eq!(children.first().unwrap().size, 8);

    // Sizes differ per child, including an empty one in the middle, so a reader that reused the previous size or
    // skipped a fixed stride would land off the next header.
    let sizes: [usize; 5] = [8, 24, 16, 0, 40];
    let five: Vec<u8> = sizes
      .iter()
      .enumerate()
      .map(|(id, size)| new_chunk_bytes(id as u32, &vec![0xFF; *size]))
      .collect::<Vec<_>>()
      .concat();
    let children: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_bytes(&five)?.get_children_cloned()?;

    assert_eq!(children.len(), 5, "Expect five chunks");
    assert_eq!(
      children.iter().map(|child| child.size).collect::<Vec<u64>>(),
      sizes.iter().map(|size| *size as u64).collect::<Vec<u64>>()
    );

    Ok(())
  }
}
