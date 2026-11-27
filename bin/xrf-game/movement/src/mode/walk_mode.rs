use std::time::Duration;

use xrf_error::XrfResult;
use xrf_math::glam::{Vec2, Vec3};
use xrf_physics::{PhysicsActorId, PhysicsActorSettings, PhysicsActorState, PhysicsGround, PhysicsWorld};

use crate::camera::movement_camera::MovementCamera;
use crate::camera::movement_heading::MovementHeading;
use crate::input::movement_action::MovementAction;
use crate::input::movement_input_state::MovementInputState;
use crate::mode::walk_state::WalkState;

/// On foot, as X-Ray's actor moves: the mouse turns the view as in flight; forward, back and sideways run along the
/// ground unless accel is held, which walks; a toggled sprint runs faster forward; up jumps; the physics world keeps
/// the actor on the level.
///
/// Speeds follow X-Ray's: the control acceleration is `walk_accel` times the pace's factors
/// (`Actor_Movement.cpp`, `g_cl_CheckControls`), and the actor's speed is held to a tenth of it
/// (`PHMovementControl.cpp`, `SetMaximumVelocity`), reached at that acceleration. The defaults are Anomaly's
/// `[actor]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WalkMode {
  /// The control acceleration of a walk, m/s²: `walk_accel`.
  pub walk_accel: f32,
  /// What running multiplies it by: `run_coef`, and `run_back_coef` backwards.
  pub run: f32,
  pub run_back: f32,
  /// What walking backwards multiplies it by: `walk_back_coef`.
  pub walk_back: f32,
  /// What sprinting multiplies a forward run by: `sprint_koef`.
  pub sprint: f32,
  /// How much of the acceleration steers the actor in the air: `air_control_param`.
  pub air_control: f32,
  /// Metres a second up a jump starts at: `jump_speed`.
  pub jump_speed: f32,
  /// Radians a count of the mouse turns by.
  pub sensitivity: f32,
  /// How high the eye stands above the feet, metres: `camera_height_factor` times the box's height less X-Ray's 5 cm
  /// (`ActorCameras.cpp`).
  pub eye_height: f32,
  pub actor: PhysicsActorSettings,
}

impl Default for WalkMode {
  fn default() -> Self {
    let actor: PhysicsActorSettings = PhysicsActorSettings::default();

    Self {
      walk_accel: 11.5,
      run: 2.41,
      run_back: 1.33,
      walk_back: 0.61,
      sprint: 2.08,
      air_control: 0.02,
      jump_speed: 6.85,
      sensitivity: 0.0025,
      eye_height: 0.92 * (actor.height - 0.05),
      actor,
    }
  }
}

impl WalkMode {
  /// What X-Ray holds the actor's speed to: a tenth of its control acceleration.
  const SPEED_PER_ACCELERATION: f32 = 0.1;

  /// Turns `camera` by the mouse's counts since the last step, then moves the actor by what is held for `step` in
  /// `physics`, putting one there at the camera's feet first where `state` holds none; the camera ends at its eye.
  ///
  /// # Errors
  ///
  /// Returns an error when the physics world refuses the actor or its move.
  pub(crate) fn step(
    &self,
    camera: &mut MovementCamera,
    input: &mut MovementInputState,
    state: &mut WalkState,
    physics: &mut PhysicsWorld,
    step: Duration,
  ) -> XrfResult {
    let look: Vec2 = input.take_look() * self.sensitivity;
    // The mouse moved right turns right and moved down looks down.
    let heading: MovementHeading = camera.get_heading().turned(look.x, look.y);
    let actor: PhysicsActorId = match state.actor {
      Some(actor) => actor,
      None => {
        let actor: PhysicsActorId = physics.create_actor(&self.actor, camera.position - Vec3::Y * self.eye_height)?;

        *state = WalkState {
          actor: Some(actor),
          ..WalkState::default()
        };

        actor
      }
    };
    let seconds: f32 = step.as_secs_f32();
    let before: PhysicsActorState = physics.get_actor(actor)?;
    let acceleration: f32 = self.get_acceleration(input);
    // Along the ground, whatever the pitch: yaw turns `+z` towards `+x`, so the right of a heading is its forward
    // turned a quarter further.
    let (sin, cos): (f32, f32) = heading.yaw.sin_cos();
    let forward: Vec3 = Vec3::new(sin, 0.0, cos);
    let right: Vec3 = Vec3::new(cos, 0.0, -sin);
    let direction: Vec3 = (forward * input.get_axis(MovementAction::Forward, MovementAction::Back)
      + right * input.get_axis(MovementAction::Right, MovementAction::Left))
    .normalize_or_zero();
    let wanted: Vec3 = direction * acceleration * Self::SPEED_PER_ACCELERATION;
    // The ground speeds the actor up and slows it down at its acceleration; the air barely steers it.
    let grip: f32 = match before.ground {
      PhysicsGround::Standing => 1.0,
      PhysicsGround::Steep | PhysicsGround::Air => self.air_control,
    };
    let change: Vec3 = wanted - state.ground_velocity;
    // Stopping slows at least as hard as a run speeds up.
    let most: f32 = acceleration.max(self.walk_accel * self.run) * grip * seconds;

    state.ground_velocity += change.clamp_length_max(most);
    // A jump leaves the ground rising, which the engine may still count as standing for a tick: standing resets the
    // fall only while it is not rising.
    state.fall = match before.ground {
      PhysicsGround::Standing if state.fall <= 0.0 => {
        if input.is_held(MovementAction::Up) {
          self.jump_speed
        } else {
          0.0
        }
      }
      _ => state.fall + PhysicsWorld::GRAVITY.y * seconds,
    };

    let after: PhysicsActorState = physics.move_actor(actor, state.ground_velocity + Vec3::Y * state.fall, seconds)?;

    *camera = MovementCamera::facing(after.feet + Vec3::Y * self.eye_height, heading);

    Ok(())
  }

  /// The control acceleration of what is held, as X-Ray scales `walk_accel`: running unless accel is held, the back
  /// factors going backwards, and sprinting forward while it is toggled on, which stopping or going back turns off.
  fn get_acceleration(&self, input: &mut MovementInputState) -> f32 {
    let forward: f32 = input.get_axis(MovementAction::Forward, MovementAction::Back);

    if forward <= 0.0 {
      input.stop_sprinting();
    }

    let is_running: bool = !input.is_held(MovementAction::Accel);
    let is_backwards: bool = forward < 0.0;
    let pace: f32 = if is_running && is_backwards {
      self.run_back
    } else if is_running {
      self.run
    } else if is_backwards {
      self.walk_back
    } else {
      1.0
    };
    let sprint: f32 = if input.is_sprinting() { self.sprint } else { 1.0 };

    self.walk_accel * pace * sprint
  }
}
