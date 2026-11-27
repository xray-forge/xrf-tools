use serde::{Deserialize, Serialize};
use xrf_game_movement::MovementCommand;

use crate::state::game_command::GameCommand;

/// A command as a recording keeps it: the game's commands that are plain values. A level change hands the loop a
/// level built beside it, which no line holds.
// todo: record a level change by the level's name, the replay opening that level again, once the game changes levels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameRecordedCommand {
  Movement(MovementCommand),
}

impl GameRecordedCommand {
  /// `command` as a recording keeps it; none for one it cannot keep.
  pub fn of(command: &GameCommand) -> Option<Self> {
    match command {
      GameCommand::Movement(command) => Some(Self::Movement(*command)),
      GameCommand::ReplaceLevel(_) => None,
    }
  }
}

impl From<GameRecordedCommand> for GameCommand {
  fn from(command: GameRecordedCommand) -> Self {
    match command {
      GameRecordedCommand::Movement(command) => Self::Movement(command),
    }
  }
}
