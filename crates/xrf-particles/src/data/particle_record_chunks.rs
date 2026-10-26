use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};

/// Reads the chunks an effect or a group is made of, naming the one that fails.
pub(crate) struct ParticleRecordChunks;

impl ParticleRecordChunks {
  /// Reads a chunk the record cannot do without, naming it whether it is missing or malformed.
  pub fn read_required<D: ChunkDataSource, R>(
    chunks: &[ChunkReader<D>],
    id: u32,
    name: &str,
    read: impl FnOnce(&mut ChunkReader<D>) -> XrfResult<R>,
  ) -> XrfResult<R> {
    ChunkReader::find_required_by_id(chunks, id)
      .and_then(|mut chunk| read(&mut chunk))
      .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle {name} chunk: {error}")))
  }

  /// Reads a chunk the record may omit: absent is `None`, present but malformed is an error naming it.
  pub fn read_optional<D: ChunkDataSource, R>(
    chunks: &[ChunkReader<D>],
    id: u32,
    name: &str,
    read: impl FnOnce(&mut ChunkReader<D>) -> XrfResult<R>,
  ) -> XrfResult<Option<R>> {
    ChunkReader::find_optional_by_id(chunks, id)
      .map(|mut chunk| read(&mut chunk))
      .transpose()
      .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle {name} chunk: {error}")))
  }
}
