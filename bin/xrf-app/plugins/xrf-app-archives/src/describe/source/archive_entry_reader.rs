use xrf_chunk::{ChunkReadWrite, ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_spawn::{SpawnFileKind, SpawnHeaderChunk};

use crate::describe::source::ArchiveEntryContainer;

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

  /// Reads a value framed at the start of the entry rather than inside a chunk of it.
  ///
  /// Some X-Ray files are not containers at all: `level.cform` and `level.ai` are a header the engine casts its file
  /// pointer onto, with the payload streamed behind it.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry holds fewer bytes than the value occupies.
  pub fn read_leading<C: ChunkReadWrite>(&mut self) -> XrfResult<C> {
    match self {
      Self::Sliced(reader) => C::read::<XRayByteOrder, _>(reader),
      Self::Held(reader) => C::read::<XRayByteOrder, _>(reader),
    }
  }

  /// Reads the whole entry as a container that walks itself.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry is not that container.
  pub fn read_container<C: ArchiveEntryContainer>(&mut self) -> XrfResult<C> {
    match self {
      Self::Sliced(reader) => C::read_container(reader),
      Self::Held(reader) => C::read_container(reader),
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

  /// The header that makes the entry a root spawn, or `None` where it carries none, by
  /// [`SpawnFileKind::read_root_header`].
  ///
  /// # Errors
  ///
  /// Returns an error when the entry is not a chunked container.
  pub fn read_spawn_header(&mut self) -> XrfResult<Option<SpawnHeaderChunk>> {
    Ok(match self {
      Self::Sliced(reader) => SpawnFileKind::read_root_header::<XRayByteOrder, _>(&reader.read_children()?),
      Self::Held(reader) => SpawnFileKind::read_root_header::<XRayByteOrder, _>(&reader.read_children()?),
    })
  }
}
