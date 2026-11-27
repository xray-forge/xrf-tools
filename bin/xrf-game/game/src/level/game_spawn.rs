use xrf_game_graph::GameGraph;
use xrf_spawn::SpawnPlacedObjects;

/// What a new game is built from, read from the tree's spawn in one pass: every object it places and the game graph
/// they stand on.
pub struct GameSpawn {
  pub placed: SpawnPlacedObjects,
  pub graph: GameGraph,
}
