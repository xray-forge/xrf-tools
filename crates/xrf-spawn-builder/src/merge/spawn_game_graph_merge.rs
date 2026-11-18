use xrf_spawn::SpawnGraphsChunk;

use crate::merge::spawn_skipped_connection::SpawnSkippedConnection;

/// The game graph the merge gives: the graph chunk of a root spawn, its GUID derived from its content, and the
/// connections it left out.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpawnGameGraphMerge {
  pub graph: SpawnGraphsChunk,
  pub skipped: Vec<SpawnSkippedConnection>,
}
