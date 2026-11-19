use xrf_level::LevelGameFile;
use xrf_spawn::{LevelSpawnFile, SpawnSectionClasses};

use crate::graph::spawn_removed_graph_point::SpawnRemovedGraphPoint;
use crate::merge::spawn_merge_level::SpawnMergeLevel;

/// One level as the root spawn build takes it: its merge input, entities, patrols where it has a `level.game`, and
/// the graph points left out, for the report.
pub(crate) struct SpawnRootLevel<'a> {
  pub merge: SpawnMergeLevel<'a>,
  pub spawn: &'a LevelSpawnFile,
  /// The class each section of `spawn` is read as.
  pub classes: &'a SpawnSectionClasses,
  pub game: Option<&'a LevelGameFile>,
  pub removed: &'a [SpawnRemovedGraphPoint],
}
