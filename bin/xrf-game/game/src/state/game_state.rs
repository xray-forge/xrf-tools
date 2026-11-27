use xrf_game_loop::{LoopState, LoopSystem, LoopTime};
use xrf_game_movement::Movement;

use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The game's systems put together, as the game loop steps them: each command goes to its system, and the systems
/// step in this order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameState {
  pub movement: Movement,
}

impl LoopSystem for GameState {
  type Command = GameCommand;

  fn apply(&mut self, command: GameCommand) {
    match command {
      GameCommand::Movement(command) => self.movement.apply(command),
    }
  }

  fn step(&mut self, time: &LoopTime) {
    self.movement.step(time);
  }
}

impl LoopState for GameState {
  type Frame = GameFrame;

  fn get_frame(&self) -> GameFrame {
    GameFrame {
      camera: *self.movement.get_camera(),
    }
  }
}
