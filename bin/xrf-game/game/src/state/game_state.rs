use xrf_game_loop::{LoopState, LoopSystem, LoopTime};
use xrf_game_movement::Movement;
use xrf_game_objects::ObjectRegistry;

use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The game's systems put together, the composition root the game loop steps: each command goes to its system, the
/// systems step in this order, and each is handed what it needs of the rest from here, as its context, never looking it
/// up. The object registry is the store they share; adding a system is a field, a command route, and a line in the step
/// below that builds its context.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameState {
  /// Every object of the game, which systems are handed the tables of they read or write.
  pub objects: ObjectRegistry,
  pub movement: Movement,
}

impl LoopState for GameState {
  type Command = GameCommand;
  type Frame = GameFrame;

  fn apply(&mut self, command: GameCommand) {
    match command {
      GameCommand::Movement(command) => self.movement.apply(command),
    }
  }

  fn step(&mut self, time: &LoopTime) {
    self.movement.step(time, ());
  }

  fn get_frame(&self) -> GameFrame {
    GameFrame {
      camera: *self.movement.get_camera(),
      objects: u32::try_from(self.objects.get_records().len()).unwrap_or(u32::MAX),
    }
  }
}
