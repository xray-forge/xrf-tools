use xrf_spawn::SpawnGraphsChunk;

use crate::merge::spawn_skipped_connection::SpawnSkippedConnection;

/// The merged game graph chunk of a root spawn, with the connections the merge left out.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpawnGameGraphMerge {
  pub graph: SpawnGraphsChunk,
  pub skipped: Vec<SpawnSkippedConnection>,
}
