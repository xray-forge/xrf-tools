use xrf_game_loop::{LoopSystem, LoopTime};

use crate::camera::movement_camera::MovementCamera;
use crate::input::movement_input_state::MovementInputState;
use crate::movement_command::MovementCommand;

/// The player's movement: the camera, and what they hold and how far they looked, which a movement mode (fly now,
/// walk on physics later) turns into motion each step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Movement {
  camera: MovementCamera,
  input: MovementInputState,
}

impl Movement {
  /// Movement with its camera standing at `camera`.
  pub fn new(camera: MovementCamera) -> Self {
    Self {
      camera,
      input: MovementInputState::default(),
    }
  }

  pub fn get_camera(&self) -> &MovementCamera {
    &self.camera
  }

  pub fn get_input(&self) -> &MovementInputState {
    &self.input
  }
}

impl LoopSystem for Movement {
  type Command = MovementCommand;

  fn apply(&mut self, command: MovementCommand) {
    match command {
      MovementCommand::Input(input) => self.input.apply(input.kind),
      MovementCommand::SetCamera(camera) => self.camera = camera,
    }
  }

  // No movement mode moves the camera yet: fly mode is the first.
  fn step(&mut self, _: &LoopTime) {}
}
