use xrf_game_graph::{GameGraph, GamePatrols};
use xrf_spawn::SpawnPlacedObjects;

/// What a new game is built from, read from the tree's spawn in one pass: every object it places, the game graph they
/// stand on, and the patrol paths laid over its levels.
pub struct GameSpawn {
  pub placed: SpawnPlacedObjects,
  pub graph: GameGraph,
  pub patrols: GamePatrols,
}
