use xrf_level::LevelGameFile;
use xrf_spawn::AlifeObject;

use crate::graph::spawn_removed_graph_point::SpawnRemovedGraphPoint;
use crate::merge::spawn_merge_level::SpawnMergeLevel;

/// One level as the root spawn build takes it: its merge input, entities, patrols where it has a `level.game`, and
/// the graph points left out, for the report.
pub struct SpawnRootLevel<'a> {
  pub merge: SpawnMergeLevel<'a>,
  /// Every object of its `level.spawn` in order, typed as its section's class.
  pub objects: &'a [AlifeObject],
  pub game: Option<&'a LevelGameFile>,
  pub removed: &'a [SpawnRemovedGraphPoint],
}
