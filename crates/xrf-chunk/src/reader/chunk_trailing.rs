use xrf_error::XrfError;

use crate::source::chunk_data_source::ChunkDataSource;

/// Bytes after the last well-formed child of a chunk, with the error a strict walk raised on them.
///
/// Produced by [`crate::ChunkReader::read_children_with_trailing`], which passes no judgement. Whether trailing bytes
/// are tolerable is a question about a specific format — X-Ray loaders differ on whether a declared chunk size even
/// bounds the payload read — and this crate knows no format. A caller either accounts for these bytes against its own
/// rules or returns [`Self::error`].
pub struct ChunkTrailing<T: ChunkDataSource> {
  /// Offset of the first unaccounted byte, in the coordinates of the reader that produced it.
  pub position: u64,
  pub size: u64,
  /// A window over the unaccounted bytes, cut from the same source.
  pub data: T,
  /// What [`crate::ChunkReader::read_children`] would have failed with, kept so a caller that cannot account for the
  /// bytes reports the same thing a strict walk always did.
  pub error: XrfError,
}
