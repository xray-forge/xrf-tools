use xrf_game_loop::{LoopSystem, LoopTime};

use crate::camera::movement_camera::MovementCamera;
use crate::input::movement_input_state::MovementInputState;
use crate::mode::movement_footing::MovementFooting;
use crate::mode::movement_mode::MovementMode;
use crate::mode::movement_modes::MovementModes;
use crate::mode::walk_state::WalkState;
use crate::movement_command::MovementCommand;
use crate::movement_context::MovementContext;

/// The player's movement: the camera, what they hold and how far they looked, the mode they move in and how each mode
/// moves, and what walking keeps between steps (its actor in the physics world).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Movement {
  camera: MovementCamera,
  input: MovementInputState,
  mode: MovementMode,
  modes: MovementModes,
  walk: WalkState,
}

impl Movement {
  /// Movement in `mode`, each mode moving as `modes` sets, its camera standing at `camera`.
  pub fn new(camera: MovementCamera, mode: MovementMode, modes: MovementModes) -> Self {
    Self {
      camera,
      input: MovementInputState::default(),
      mode,
      modes,
      walk: WalkState::default(),
    }
  }

  pub fn get_camera(&self) -> &MovementCamera {
    &self.camera
  }

  pub fn get_input(&self) -> &MovementInputState {
    &self.input
  }

  pub fn get_mode(&self) -> MovementMode {
    self.mode
  }

  pub fn get_modes(&self) -> &MovementModes {
    &self.modes
  }

  pub fn get_walk(&self) -> &WalkState {
    &self.walk
  }

  /// What the walking actor stood on and how it moved at the last step; none in flight.
  pub fn get_footing(&self) -> Option<MovementFooting> {
    self.walk.footing
  }
}

impl LoopSystem for Movement {
  type Command = MovementCommand;
  type Context<'a> = MovementContext<'a>;

  fn apply(&mut self, command: MovementCommand) {
    match command {
      MovementCommand::Input(input) => self.input.apply(input.kind),
      MovementCommand::SetCamera(camera) => self.camera = camera,
      MovementCommand::ToggleMode => self.mode = self.mode.toggled(),
    }
  }

  /// Flying lets go of the actor walking left in the physics world; walking puts one there where it has none.
  fn step(&mut self, time: &LoopTime, MovementContext { physics }: MovementContext<'_>) {
    match self.mode {
      MovementMode::Fly => {
        if let Some(actor) = self.walk.actor
          && let Err(error) = physics.remove_actor(actor)
        {
          log::error!("The actor did not leave the physics world: {error}");
        }

        self.walk = WalkState::default();
        self.modes.fly.step(&mut self.camera, &mut self.input, time.step);
      }
      MovementMode::Walk => {
        if let Err(error) = self
          .modes
          .walk
          .step(&mut self.camera, &mut self.input, &mut self.walk, physics, time.step)
        {
          log::error!("The actor did not walk: {error}");
        }
      }
    }
  }
}
