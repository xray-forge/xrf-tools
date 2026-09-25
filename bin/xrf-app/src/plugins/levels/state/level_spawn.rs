use xrf_spawn::AlifeObject;

/// The objects the game spawns on a level, out of its one spawn for every level: what a level's lamps, and any other
/// spawned thing a view shows, are read from.
pub struct LevelSpawn {
  pub objects: Vec<AlifeObject>,
}
