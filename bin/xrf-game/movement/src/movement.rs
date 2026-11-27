use xrf_game_loop::{LoopSystem, LoopTime};

use crate::camera::movement_camera::MovementCamera;
use crate::input::movement_input_state::MovementInputState;
use crate::mode::movement_mode::MovementMode;
use crate::movement_command::MovementCommand;

/// The player's movement: the camera, what they hold and how far they looked, and the mode that turns that into motion
/// each step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Movement {
  camera: MovementCamera,
  input: MovementInputState,
  mode: MovementMode,
}

impl Movement {
  /// Movement in `mode`, its camera standing at `camera`.
  pub fn new(camera: MovementCamera, mode: MovementMode) -> Self {
    Self {
      camera,
      input: MovementInputState::default(),
      mode,
    }
  }

  pub fn get_camera(&self) -> &MovementCamera {
    &self.camera
  }

  pub fn get_input(&self) -> &MovementInputState {
    &self.input
  }

  pub fn get_mode(&self) -> &MovementMode {
    &self.mode
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

  fn step(&mut self, time: &LoopTime) {
    match &self.mode {
      MovementMode::Fly(fly) => fly.step(&mut self.camera, &mut self.input, time.step),
    }
  }
}
