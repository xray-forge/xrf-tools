use xrf_level::LevelGameFile;
use xrf_spawn::LevelSpawnFile;

use crate::spawn::spawn_merge_level::SpawnMergeLevel;

/// One level as the root spawn build takes it: what its game graph merges from, its entities, and its patrol paths
/// where it has a `level.game`.
pub struct SpawnRootLevel<'a> {
  pub merge: SpawnMergeLevel<'a>,
  pub spawn: &'a LevelSpawnFile,
  pub game: Option<&'a LevelGameFile>,
}
