use xrf_game_movement::MovementCommand;

use crate::state::game_level_entry::GameLevelEntry;

/// Every change from outside the game's state, routed to the system it is for.
#[derive(Debug)]
pub enum GameCommand {
  Movement(MovementCommand),
  /// Plays another level from the next tick: its state in place of the one played, the player at its start, what the
  /// game holds kept.
  ReplaceLevel(Box<GameLevelEntry>),
}
