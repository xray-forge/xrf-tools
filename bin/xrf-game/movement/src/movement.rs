use xrf_error::XrfResult;
use xrf_game_loop::{LoopSystem, LoopTime};
use xrf_math::glam::Vec3;

use crate::camera::movement_camera::MovementCamera;
use crate::camera::movement_heading::MovementHeading;
use crate::event::movement_event::MovementEvent;
use crate::input::movement_input_state::MovementInputState;
use crate::mode::movement_footing::MovementFooting;
use crate::mode::movement_mode::MovementMode;
use crate::mode::movement_modes::MovementModes;
use crate::mode::walk_state::WalkState;
use crate::mode::walk_strides::WalkStrides;
use crate::movement_command::MovementCommand;
use crate::movement_context::MovementContext;

/// The player's movement: the camera, which way presentation faces it and what the player holds, the mode they move in
/// and how each mode moves, what walking keeps between steps (its actor in the physics world, its strides), and how
/// fast the camera went at the last step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Movement {
  camera: MovementCamera,
  heading: MovementHeading,
  input: MovementInputState,
  mode: MovementMode,
  modes: MovementModes,
  walk: WalkState,
  strides: WalkStrides,
  /// Metres a second the camera moved at over the last step; none after it was stood somewhere.
  velocity: Vec3,
  /// Whether the camera was stood somewhere since the last step, which the step's velocity does not count.
  is_placed: bool,
}

impl Movement {
  /// Movement in `mode`, each mode moving as `modes` sets, its camera standing at `camera`, facing as it does.
  pub fn new(camera: MovementCamera, mode: MovementMode, modes: MovementModes) -> Self {
    Self {
      heading: camera.get_heading(),
      camera,
      input: MovementInputState::default(),
      mode,
      modes,
      walk: WalkState::default(),
      strides: WalkStrides::default(),
      velocity: Vec3::ZERO,
      is_placed: false,
    }
  }

  pub fn get_camera(&self) -> &MovementCamera {
    &self.camera
  }

  /// Which way presentation last faced the player.
  pub fn get_heading(&self) -> MovementHeading {
    self.heading
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

  /// Metres a second the camera moved at over the last step, which presentation carries it forward by.
  pub fn get_velocity(&self) -> Vec3 {
    self.velocity
  }

  /// What the walking actor stood on and how it moved at the last step; none in flight.
  pub fn get_footing(&self) -> Option<MovementFooting> {
    self.walk.footing
  }

  /// Where the player's feet stand: below the eye by its height while walking, at the camera in flight.
  pub fn get_feet(&self) -> Vec3 {
    match self.walk.actor {
      Some(_) => self.camera.position - Vec3::Y * self.walk.eye,
      None => self.camera.position,
    }
  }

  /// Moves into another level's physics world, standing at `camera`: what walking kept of the level left (its actor
  /// in that world, its strides) goes with it, and the next walking step puts the actor into the new world.
  pub fn enter_level(&mut self, camera: MovementCamera) {
    self.stand(camera);
    self.walk = WalkState::default();
    self.strides = WalkStrides::default();
  }

  fn stand(&mut self, camera: MovementCamera) {
    self.camera = camera;
    self.heading = camera.get_heading();
    self.is_placed = true;
  }
}

impl LoopSystem for Movement {
  type Command = MovementCommand;
  type Context<'a> = MovementContext<'a>;

  fn apply(&mut self, command: MovementCommand) {
    match command {
      MovementCommand::Input(input) => self.input.apply(input.kind),
      MovementCommand::Face(heading) => self.heading = heading,
      MovementCommand::SetCamera(camera) => self.stand(camera),
      MovementCommand::ToggleMode => self.mode = self.mode.toggled(),
    }
  }

  /// Faces the camera where presentation turned it, then moves it: flying lets go of the actor walking left in the
  /// physics world; walking puts one there where it has none and tells each stride the feet finish. The player's record
  /// then stands where its feet do.
  fn step(
    &mut self,
    time: &LoopTime,
    MovementContext {
      physics,
      player,
      events,
    }: MovementContext<'_>,
  ) -> XrfResult<()> {
    let before: Vec3 = self.camera.position;

    self.camera = MovementCamera::facing(self.camera.position, self.heading);

    match self.mode {
      MovementMode::Fly => {
        if let Some(actor) = self.walk.actor.take() {
          physics.remove_actor(actor)?;
        }

        self.walk = WalkState::default();
        self.modes.fly.step(&mut self.camera, &self.input, time.step);
      }
      MovementMode::Walk => {
        self
          .modes
          .walk
          .step(&mut self.camera, &mut self.input, &mut self.walk, physics, time.step)?;
      }
    }

    if let Some(step) = self.strides.take(self.walk.footing.as_ref()) {
      events.push(MovementEvent::Step(step));
    }

    self.velocity = if std::mem::take(&mut self.is_placed) || time.step.is_zero() {
      Vec3::ZERO
    } else {
      (self.camera.position - before) / time.step.as_secs_f32()
    };

    // todo: write the player's facing into its record, in the spawn's angles, once a save or a system reads it.
    player.position = self.get_feet();

    Ok(())
  }
}
