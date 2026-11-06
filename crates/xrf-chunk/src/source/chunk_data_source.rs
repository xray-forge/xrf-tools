use std::io::{Read, Result as IoResult, SeekFrom};
use std::ops::RangeBounds;

use byteorder::ReadBytesExt;

/// A seekable window of bytes a [`crate::ChunkReader`] reads from, which can cut sub-windows for its children.
///
/// `start_pos`, `cursor_pos` and `end_pos` are in the source's own coordinates: offsets into the file for a
/// [`fileslice::FileSlice`], offsets from the window's own start for an
/// [`crate::InMemoryChunkDataSource`]. Seeking and slicing are always relative to the window's start.
pub trait ChunkDataSource: ReadBytesExt + Read + Clone {
  /// Where this window starts.
  fn start_pos(&self) -> u64;

  /// The next byte to be read.
  fn cursor_pos(&self) -> u64;

  /// Where this window ends, exclusive.
  fn end_pos(&self) -> u64;

  /// Moves the cursor, with `SeekFrom::Start(0)` at the window's first byte.
  fn set_seek(&mut self, pos: SeekFrom) -> IoResult<u64>;

  /// The cursor's offset from the window's start.
  fn get_seek(&mut self) -> IoResult<u64>;

  /// A sub-window over the same bytes, addressed from this window's start, with its own cursor at its start.
  fn slice<T: RangeBounds<u64>>(&self, range: T) -> Self;

  /// Reads exactly `count` bytes from the cursor.
  fn read_bytes(&mut self, count: usize) -> IoResult<Vec<u8>> {
    let mut buffer: Vec<u8> = vec![0; count];

    self.read_exact(&mut buffer)?;

    Ok(buffer)
  }

  /// Hands the next `count` bytes to `visit` and moves past them.
  ///
  /// For a reader that decodes a run of bytes into something else and keeps nothing of them: a source holding its bytes
  /// in memory lends them where they lie, where reading them out first would copy every vertex and index payload once
  /// more than decoding it does. This default reads them into a buffer.
  fn read_with<R>(&mut self, count: usize, visit: impl FnOnce(&[u8]) -> R) -> IoResult<R> {
    Ok(visit(&self.read_bytes(count)?))
  }

  /// [`Self::read_with`] at a window-local offset, without moving this source's own cursor.
  fn read_at_with<R>(&self, offset: u64, count: usize, visit: impl FnOnce(&[u8]) -> R) -> IoResult<R> {
    self.slice(offset..offset + count as u64).read_with(count, visit)
  }
}
