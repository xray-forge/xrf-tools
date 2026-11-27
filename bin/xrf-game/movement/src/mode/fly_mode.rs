use std::time::Duration;

use xrf_math::glam::{Vec2, Vec3};

use crate::camera::movement_camera::MovementCamera;
use crate::input::movement_action::MovementAction;
use crate::input::movement_input_state::MovementInputState;

/// A free camera, as a spectator flies: the mouse turns it, forward and back fly where it looks, sideways along its
/// right, up and down straight up and down, through everything.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlyMode {
  /// Metres a second.
  pub speed: f32,
  /// What the speed is multiplied by while the fast action is held.
  pub boost: f32,
  /// Radians a count of the mouse turns by.
  pub sensitivity: f32,
}

impl Default for FlyMode {
  fn default() -> Self {
    Self {
      speed: 10.0,
      boost: 4.0,
      sensitivity: 0.0025,
    }
  }
}

impl FlyMode {
  /// Turns `camera` by the mouse's counts since the last step, then flies it by what is held for `step`.
  pub(crate) fn step(&self, camera: &mut MovementCamera, input: &mut MovementInputState, step: Duration) {
    let look: Vec2 = input.take_look() * self.sensitivity;

    // The mouse moved right turns right and moved down looks down.
    *camera = MovementCamera::facing(camera.position, camera.get_heading().turned(look.x, look.y));

    let forward: f32 = input.get_axis(MovementAction::Forward, MovementAction::Back);
    let strafe: f32 = input.get_axis(MovementAction::Right, MovementAction::Left);
    let rise: f32 = input.get_axis(MovementAction::Up, MovementAction::Down);
    let direction: Vec3 = camera.get_forward() * forward + camera.get_right() * strafe + Vec3::Y * rise;

    if direction == Vec3::ZERO {
      return;
    }

    let speed: f32 = self.speed
      * if input.is_held(MovementAction::Fast) {
        self.boost
      } else {
        1.0
      };

    // Diagonals fly no faster than straight lines.
    camera.position += direction.normalize() * speed * step.as_secs_f32();
  }
}
