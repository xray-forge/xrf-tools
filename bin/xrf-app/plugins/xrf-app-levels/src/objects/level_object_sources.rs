use xrf_level::LevelDynamicLight;
use xrf_spawn::SpawnLevelGraph;

use crate::parts::{LevelPlacements, LevelSpawn};

/// What a level's objects are read from: its share of the spawn, the spawn's graph of it, its companion files and its
/// bundle's static lights, each but the spawn left out where it cannot be read.
#[derive(Clone, Copy)]
pub struct LevelObjectSources<'a> {
  pub spawn: &'a LevelSpawn,
  pub graph: Option<&'a SpawnLevelGraph>,
  pub placements: Option<&'a LevelPlacements>,
  pub lights: &'a [LevelDynamicLight],
}

impl<'a> LevelObjectSources<'a> {
  /// The spawn's objects alone.
  pub fn of_spawn(spawn: &'a LevelSpawn) -> Self {
    Self {
      spawn,
      graph: None,
      placements: None,
      lights: &[],
    }
  }
}
