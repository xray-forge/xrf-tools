use std::sync::Arc;

use xrf_game_graph::{GameGraph, GamePatrols};
use xrf_game_simulation::SimulationConfig;

/// What A-Life reads and never changes during a game, read once at its start and shared: the game graph offline
/// objects walk, the spawn's patrols, the simulation's configs (squads, smart terrains, start positions), and the
/// game's seed, which keys every draw the simulation makes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AlifeData {
  pub graph: Arc<GameGraph>,
  pub patrols: Arc<GamePatrols>,
  pub simulation: Arc<SimulationConfig>,
  pub seed: u64,
}
