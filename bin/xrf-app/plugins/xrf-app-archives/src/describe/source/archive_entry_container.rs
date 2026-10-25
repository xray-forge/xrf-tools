use xrf_chunk::{ChunkDataSource, ChunkReader, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_level::LevelGeomFile;
use xrf_spawn::LevelSpawnFile;

/// A container an entry can be read as wherever it sits.
pub trait ArchiveEntryContainer: Sized {
  /// Reads the container out of a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not this container.
  fn read_container<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self>;
}

impl ArchiveEntryContainer for LevelGeomFile {
  fn read_container<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Self::read_from_chunk::<XRayByteOrder, D>(reader)
  }
}

impl ArchiveEntryContainer for LevelSpawnFile {
  fn read_container<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Self::read_from_chunk::<XRayByteOrder, D>(reader)
  }
}
