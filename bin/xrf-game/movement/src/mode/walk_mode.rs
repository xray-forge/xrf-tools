use std::time::Duration;

use xrf_error::XrfResult;
use xrf_math::glam::Vec3;
use xrf_physics::{PhysicsActorId, PhysicsActorSettings, PhysicsActorState, PhysicsGround, PhysicsWorld};

use crate::camera::movement_camera::MovementCamera;
use crate::camera::movement_heading::MovementHeading;
use crate::input::movement_action::MovementAction;
use crate::input::movement_input_state::MovementInputState;
use crate::mode::movement_footing::MovementFooting;
use crate::mode::walk_pace::WalkPace;
use crate::mode::walk_stance::WalkStance;
use crate::mode::walk_state::WalkState;

/// On foot, as X-Ray's actor moves: the view faces as presentation turned it; forward, back and sideways run along the
/// ground unless accel is held, which walks; a toggled sprint runs faster forward; up jumps; down crouches, low with
/// accel held; the physics world keeps the actor on the level.
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
  /// What sprinting multiplies a run by: `sprint_koef`.
  pub sprint: f32,
  /// What crouching multiplies it by: `crouch_coef`.
  pub crouch: f32,
  /// What going sideways standing multiplies it by, walking and running: `walk_strafe_coef` and `run_strafe_coef`, 1
  /// where the configs set none, as Call of Pripyat's and Anomaly's do not.
  pub walk_strafe: f32,
  pub run_strafe: f32,
  /// How much of the acceleration steers the actor in the air: `air_control_param`.
  pub air_control: f32,
  /// Metres a second up a jump starts at: `jump_speed`.
  pub jump_speed: f32,
  /// The actor's height crouched, metres: `ph_box1`.
  pub crouch_height: f32,
  /// The actor's height crouched low, metres: `ph_box2`.
  pub low_height: f32,
  /// Where the eye stands up the actor's box: `camera_height_factor`, of the box's height less X-Ray's 5 cm
  /// (`ActorCameras.cpp`).
  pub eye_factor: f32,
  /// Metres a second the eye moves at to a new stance's height.
  pub eye_speed: f32,
  pub actor: PhysicsActorSettings,
}

impl Default for WalkMode {
  fn default() -> Self {
    Self {
      walk_accel: 11.5,
      run: 2.41,
      run_back: 1.33,
      walk_back: 0.61,
      sprint: 2.08,
      crouch: 0.45,
      walk_strafe: 1.0,
      run_strafe: 1.0,
      air_control: 0.02,
      jump_speed: 6.85,
      crouch_height: 1.34,
      low_height: 0.44,
      eye_factor: 0.92,
      // Ours: X-Ray's camera follows the box's height through its own effectors.
      eye_speed: 3.0,
      actor: PhysicsActorSettings::default(),
    }
  }
}

impl WalkMode {
  /// What X-Ray holds the actor's speed to: a tenth of its control acceleration.
  const SPEED_PER_ACCELERATION: f32 = 0.1;

  /// Metres a second the actor keeps on the ground at `factor` times `walk_accel`: a walk at 1, a run at `run`.
  pub fn get_speed(&self, factor: f32) -> f32 {
    self.walk_accel * factor * Self::SPEED_PER_ACCELERATION
  }

  /// What X-Ray takes off a box's height before it places the eye (`cammera_into_collision_shift`).
  const EYE_SHIFT: f32 = 0.05;

  /// How high the eye stands over the feet in `stance`.
  pub fn get_eye_height(&self, stance: WalkStance) -> f32 {
    self.eye_factor * (self.get_height(stance) - Self::EYE_SHIFT)
  }

  /// The actor's height in `stance`.
  pub fn get_height(&self, stance: WalkStance) -> f32 {
    match stance {
      WalkStance::Standing => self.actor.height,
      WalkStance::Crouching => self.crouch_height,
      WalkStance::CrouchingLow => self.low_height,
    }
  }

  /// Moves the actor along where `camera` faces by what is held for `step` in `physics`, putting one there at the
  /// camera's feet first where `state` holds none; the camera ends at its eye, facing as it did.
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
    let heading: MovementHeading = camera.get_heading();
    let actor: PhysicsActorId = match state.actor {
      Some(actor) => actor,
      None => {
        let eye: f32 = self.get_eye_height(WalkStance::Standing);
        let actor: PhysicsActorId = physics.create_actor(&self.actor, camera.position - Vec3::Y * eye)?;

        *state = WalkState {
          actor: Some(actor),
          eye,
          ..WalkState::default()
        };

        actor
      }
    };
    let seconds: f32 = step.as_secs_f32();
    let before: PhysicsActorState = physics.get_actor(actor)?;

    self.change_stance(input, state, physics, actor, before.ground)?;

    let acceleration: f32 = self.get_acceleration(input, state.stance);
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
    // fall only while it is not rising. A crouched actor does not jump.
    state.fall = match before.ground {
      PhysicsGround::Standing if state.fall <= 0.0 => {
        if input.is_held(MovementAction::Up) && state.stance == WalkStance::Standing {
          self.jump_speed
        } else {
          0.0
        }
      }
      _ => state.fall + PhysicsWorld::GRAVITY.y * seconds,
    };

    let after: PhysicsActorState = physics.move_actor(actor, state.ground_velocity + Vec3::Y * state.fall, seconds)?;
    let eye: f32 = self.get_eye_height(state.stance);

    state.eye += (eye - state.eye).clamp(-self.eye_speed * seconds, self.eye_speed * seconds);
    state.footing = Some(MovementFooting {
      feet: after.feet,
      ground_material: after.ground_material,
      pace: self.get_pace(input, state.stance),
      speed: state.ground_velocity.length(),
    });
    *camera = MovementCamera::facing(after.feet + Vec3::Y * state.eye, heading);

    Ok(())
  }

  /// Crouches, crouches low or stands as down and accel ask, as X-Ray's actor does: down crouches only on the ground,
  /// low with accel held; any stance changes only where the actor's new box has room.
  fn change_stance(
    &self,
    input: &MovementInputState,
    state: &mut WalkState,
    physics: &mut PhysicsWorld,
    actor: PhysicsActorId,
    ground: PhysicsGround,
  ) -> XrfResult {
    let wanted: WalkStance = if !input.is_held(MovementAction::Down) {
      WalkStance::Standing
    } else if input.is_held(MovementAction::Accel) {
      WalkStance::CrouchingLow
    } else {
      WalkStance::Crouching
    };
    let is_allowed: bool = wanted == WalkStance::Standing || ground == PhysicsGround::Standing;

    if wanted != state.stance && is_allowed && physics.reshape_actor(actor, self.get_height(wanted))? {
      state.stance = wanted;
    }

    Ok(())
  }

  /// The control acceleration of what is held, as X-Ray scales `walk_accel` (`g_cl_CheckControls`): running unless
  /// accel is held, the back factors going backwards, the crouch factor crouched, the strafe factors sideways standing,
  /// and the sprint factor while sprinting. The sprint toggle holds while forward or a side is held and the actor runs
  /// standing, and drops otherwise; sideways alone it adds nothing until forward is held again.
  fn get_acceleration(&self, input: &mut MovementInputState, stance: WalkStance) -> f32 {
    let forward: f32 = input.get_axis(MovementAction::Forward, MovementAction::Back);
    let side: f32 = input.get_axis(MovementAction::Right, MovementAction::Left);
    let is_crouched: bool = stance != WalkStance::Standing;
    let is_running: bool = !input.is_held(MovementAction::Accel);

    if !(forward > 0.0 || side != 0.0) || is_crouched || !is_running {
      input.stop_sprinting();
    }

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
    let crouch: f32 = if is_crouched { self.crouch } else { 1.0 };
    let sprint: f32 = if Self::is_sprinting(input) { self.sprint } else { 1.0 };
    let strafe: f32 = if side == 0.0 || is_crouched {
      1.0
    } else if is_running {
      self.run_strafe
    } else {
      self.walk_strafe
    };

    self.walk_accel * pace * crouch * sprint * strafe
  }

  /// Whether the sprint factor applies: toggled on and going forward or back, as X-Ray clears it for a frame of
  /// sideways alone (`Actor_Movement.cpp`, opposite keys).
  fn is_sprinting(input: &MovementInputState) -> bool {
    input.is_sprinting() && input.get_axis(MovementAction::Forward, MovementAction::Back) != 0.0
  }

  fn get_pace(&self, input: &MovementInputState, stance: WalkStance) -> WalkPace {
    if stance != WalkStance::Standing {
      WalkPace::Crouch
    } else if Self::is_sprinting(input) {
      WalkPace::Sprint
    } else if input.is_held(MovementAction::Accel) {
      WalkPace::Walk
    } else {
      WalkPace::Run
    }
  }
}
