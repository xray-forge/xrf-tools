use xrf_level::LevelGameFile;
use xrf_spawn::LevelSpawnFile;

use crate::spawn::spawn_merge_level::SpawnMergeLevel;
use crate::spawn::spawn_removed_graph_point::SpawnRemovedGraphPoint;

/// One level as the root spawn build takes it: what its game graph merges from, its entities, and its patrol paths
/// where it has a `level.game`; the graph points its graph left out, for the report.
pub struct SpawnRootLevel<'a> {
  pub merge: SpawnMergeLevel<'a>,
  pub spawn: &'a LevelSpawnFile,
  pub game: Option<&'a LevelGameFile>,
  pub removed: &'a [SpawnRemovedGraphPoint],
}
