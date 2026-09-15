use xrf_chunk::{ChunkReadWrite, ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;

/// An entry opened for reading, positionally where the subject can manage it.
pub enum ArchiveEntryReader {
  /// A file on disk, read where it sits.
  Sliced(ChunkReader),
  /// An entry held whole, because it came out of a volume.
  Held(ChunkReader<InMemoryChunkDataSource>),
}

impl ArchiveEntryReader {
  /// Bytes the entry occupies unpacked.
  pub const fn size(&self) -> u64 {
    match self {
      Self::Sliced(reader) => reader.size,
      Self::Held(reader) => reader.size,
    }
  }

  /// Every top-level chunk as its id and size, read by the headers alone.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry is not a chunked container, or its children do not account for it.
  pub fn read_sections(&mut self) -> XrfResult<Vec<(u32, u64)>> {
    let sections: Vec<(u32, u64)> = match self {
      Self::Sliced(reader) => reader.read_children()?.iter().map(|it| (it.id, it.size)).collect(),
      Self::Held(reader) => reader.read_children()?.iter().map(|it| (it.id, it.size)).collect(),
    };

    Ok(sections)
  }

  /// Reads the payload of one top-level chunk, or `None` when the entry declares no such chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry is not a chunked container, or the chunk's payload is not a `C`.
  pub fn read_chunk<C: ChunkReadWrite>(&mut self, id: u32) -> XrfResult<Option<C>> {
    match self {
      Self::Sliced(reader) => Self::read_chunk_of(reader, id),
      Self::Held(reader) => Self::read_chunk_of(reader, id),
    }
  }

  fn read_chunk_of<C: ChunkReadWrite, D: xrf_chunk::ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    id: u32,
  ) -> XrfResult<Option<C>> {
    let Some(mut chunk) = reader.read_children()?.into_iter().find(|chunk| chunk.id == id) else {
      return Ok(None);
    };

    Ok(Some(C::read::<XRayByteOrder, _>(&mut chunk)?))
  }
}
