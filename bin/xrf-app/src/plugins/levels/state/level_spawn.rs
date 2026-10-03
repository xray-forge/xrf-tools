use std::collections::HashMap;

use xrf_spawn::{AlifeObject, SpawnLevelArrival};

use crate::plugins::levels::state::LevelSpawnRelease;

/// The objects the game spawns on a level, out of its one spawn for every level: what a level's lamps, and any other
/// spawned thing a view shows, are read from.
pub struct LevelSpawn {
  /// Every object of the level, those a new game releases included.
  pub objects: Vec<AlifeObject>,
  /// Where the changers of every level leading here put the actor, which is where the level opens.
  pub arrivals: Vec<SpawnLevelArrival>,
  /// The objects a new game releases, by their index among `objects`, and why.
  pub releases: HashMap<usize, LevelSpawnRelease>,
}

impl LevelSpawn {
  /// Why a new game releases the object at `index`, or `None` for one it keeps.
  pub fn get_release(&self, index: usize) -> Option<LevelSpawnRelease> {
    self.releases.get(&index).copied()
  }

  /// The objects a new game keeps: what lights a level and where its actor stands.
  pub fn list_kept(&self) -> impl Iterator<Item = &AlifeObject> {
    self
      .objects
      .iter()
      .enumerate()
      .filter(|(index, _)| !self.releases.contains_key(index))
      .map(|(_, object)| object)
  }
}
