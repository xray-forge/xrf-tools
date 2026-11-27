use xrf_game_movement::MovementCamera;

use crate::state::game_level_state::GameLevelState;

/// A level the game moves to, built on the pool beside the running game and handed to the loop whole: its state, and
/// where the player stands on it.
#[derive(Debug)]
pub struct GameLevelEntry {
  pub level: GameLevelState,
  pub start: MovementCamera,
}
