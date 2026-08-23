use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, SeekFrom};
use std::ops::RangeBounds;
use std::sync::Arc;

use crate::source::chunk_data_source::ChunkDataSource;

/// A window onto bytes held in memory, shared with every window cut from it.
///
/// The sharing is the point. A chunked format is read by slicing a parent into children, and the particles library alone
/// cuts 921 of them, so a source that owned its bytes copied the whole file once per nesting level. Windows carry offsets
/// into one `Arc`, which makes both `slice` and `clone` free — the second matters because cloning a reader is otherwise a
/// silent copy of however many megabytes the asset happens to be.
///
/// Positions are window-local, as they were when this held a `Cursor`: `start_pos` is always 0 and the cursor counts from
/// the window's own beginning, so a child reports offsets relative to itself rather than to the file.
#[derive(Clone)]
pub struct InMemoryChunkDataSource {
  data: Arc<Vec<u8>>,
  start: usize,
  end: usize,
  position: u64,
}

impl InMemoryChunkDataSource {
  /// Copies `buf` into a seekable in-memory source.
  ///
  /// Prefer [`Self::from_vec`] when the bytes are already owned — this copies them.
  pub fn from_buffer(buf: &[u8]) -> Self {
    Self::from_vec(buf.to_vec())
  }

  /// Wraps owned bytes without copying.
  ///
  /// The seam for reading assets out of a mounted VFS, where the read already produced an owned buffer per entry and a
  /// copy would double the allocation of every archived mesh and level read.
  pub fn from_vec(buf: Vec<u8>) -> Self {
    let end: usize = buf.len();

    Self {
      data: Arc::new(buf),
      start: 0,
      end,
      position: 0,
    }
  }

  /// Count of bytes left to read from the current cursor position.
  pub fn len(&self) -> u64 {
    self.length().saturating_sub(self.position)
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Size of this window, whatever the cursor is doing.
  fn length(&self) -> u64 {
    (self.end - self.start) as u64
  }

  /// Absolute index into the shared buffer for a window-local offset.
  fn index_of(&self, offset: u64) -> usize {
    // Saturating rather than wrapping: a cursor seeked past the end reads nothing instead of addressing another window.
    self.start.saturating_add(offset.min(self.length()) as usize)
  }
}

impl ChunkDataSource for InMemoryChunkDataSource {
  fn start_pos(&self) -> u64 {
    0
  }

  fn cursor_pos(&self) -> u64 {
    self.position
  }

  fn end_pos(&self) -> u64 {
    self.length()
  }

  /// Seeks within this window, as `Cursor` did: past the end is allowed, before the start is not.
  fn set_seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
    let target: i64 = match pos {
      SeekFrom::Start(offset) => {
        self.position = offset;

        return Ok(offset);
      }
      SeekFrom::End(offset) => self.length() as i64 + offset,
      SeekFrom::Current(offset) => self.position as i64 + offset,
    };

    if target < 0 {
      return Err(IoError::new(
        ErrorKind::InvalidInput,
        "invalid seek to a negative position",
      ));
    }

    self.position = target as u64;

    Ok(self.position)
  }

  fn get_seek(&mut self) -> IoResult<u64> {
    Ok(self.position)
  }

  /// Cuts a sub-window, addressed in this window's own coordinates, sharing the same bytes.
  fn slice<T: RangeBounds<u64>>(&self, range: T) -> Self {
    let start: u64 = match range.start_bound() {
      std::ops::Bound::Included(&start) => start,
      std::ops::Bound::Excluded(&start) => start.saturating_add(1),
      std::ops::Bound::Unbounded => 0,
    };

    let end: u64 = match range.end_bound() {
      std::ops::Bound::Included(&end) => end.saturating_add(1),
      std::ops::Bound::Excluded(&end) => end,
      std::ops::Bound::Unbounded => self.length(),
    };

    // Clamped rather than indexed: a corrupt chunk header must answer an empty window, never panic inside a reader.
    let start: usize = self.index_of(start);
    let end: usize = self.index_of(end).max(start);

    Self {
      data: Arc::clone(&self.data),
      start,
      end,
      position: 0,
    }
  }
}

impl Read for InMemoryChunkDataSource {
  fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
    let available: usize = self.len() as usize;
    let count: usize = available.min(buf.len());
    let from: usize = self.index_of(self.position);

    buf[..count].copy_from_slice(&self.data[from..from + count]);
    self.position += count as u64;

    Ok(count)
  }
}

#[cfg(test)]
mod tests {
  use std::io::{Read, SeekFrom};
  use std::sync::Arc;

  use crate::source::chunk_data_source::ChunkDataSource;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;

  fn source() -> InMemoryChunkDataSource {
    InMemoryChunkDataSource::from_vec((0u8..32).collect())
  }

  #[test]
  fn reads_and_reports_window_local_positions() {
    let mut source: InMemoryChunkDataSource = source();
    let mut buffer: [u8; 4] = [0; 4];

    assert_eq!(source.start_pos(), 0);
    assert_eq!(source.end_pos(), 32);
    assert_eq!(source.len(), 32);

    source.read_exact(&mut buffer).unwrap();

    assert_eq!(buffer, [0, 1, 2, 3]);
    assert_eq!(source.cursor_pos(), 4);
    assert_eq!(source.len(), 28);
  }

  #[test]
  fn slices_in_its_own_coordinates() {
    let mut child: InMemoryChunkDataSource = source().slice(8..12);
    let mut buffer: [u8; 4] = [0; 4];

    // A child counts from its own beginning, which is what a chunk reader reports as its position.
    assert_eq!(child.start_pos(), 0);
    assert_eq!(child.end_pos(), 4);

    child.read_exact(&mut buffer).unwrap();

    assert_eq!(buffer, [8, 9, 10, 11]);
    assert_eq!(child.read(&mut buffer).unwrap(), 0);
  }

  #[test]
  fn slices_a_slice_relative_to_the_parent_window() {
    let mut grandchild: InMemoryChunkDataSource = source().slice(8..16).slice(2..4);
    let mut buffer: [u8; 2] = [0; 2];

    grandchild.read_exact(&mut buffer).unwrap();

    assert_eq!(buffer, [10, 11]);
  }

  #[test]
  fn shares_one_buffer_across_every_window() {
    let parent: InMemoryChunkDataSource = source();
    let children: Vec<InMemoryChunkDataSource> = (0..8).map(|index| parent.slice(index..index + 2)).collect();
    let clones: Vec<InMemoryChunkDataSource> = children.clone();

    // The whole point of the shape: slicing and cloning must not copy the bytes.
    assert_eq!(Arc::strong_count(&parent.data), 1 + children.len() + clones.len());
  }

  #[test]
  fn seeks_like_a_cursor() {
    let mut source: InMemoryChunkDataSource = source();

    assert_eq!(source.set_seek(SeekFrom::Start(8)).unwrap(), 8);
    assert_eq!(source.set_seek(SeekFrom::Current(4)).unwrap(), 12);
    assert_eq!(source.set_seek(SeekFrom::End(-2)).unwrap(), 30);
    // Past the end is allowed and reads nothing, as a cursor over a vector does.
    assert_eq!(source.set_seek(SeekFrom::Start(64)).unwrap(), 64);
    assert_eq!(source.read(&mut [0; 4]).unwrap(), 0);
    assert!(source.set_seek(SeekFrom::Current(-128)).is_err());
  }

  #[test]
  fn answers_an_empty_window_for_a_range_beyond_the_end() {
    // A corrupt chunk header must not panic the reader it is being read through.
    let child: InMemoryChunkDataSource = source().slice(64..128);

    assert_eq!(child.end_pos(), 0);
    assert!(child.is_empty());
  }
}
