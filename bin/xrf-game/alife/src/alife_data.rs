use std::sync::Arc;

use xrf_game_graph::GameGraph;
use xrf_game_simulation::SimulationConfig;

/// What A-Life reads and never changes, read once a session and shared: the game graph offline objects walk, and the
/// simulation's configs (squads, smart terrains, start positions).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AlifeData {
  pub graph: Arc<GameGraph>,
  pub simulation: Arc<SimulationConfig>,
}
