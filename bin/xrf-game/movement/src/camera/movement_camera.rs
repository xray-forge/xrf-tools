use serde::{Deserialize, Serialize};
use xrf_game_loop::LoopBlend;
use xrf_math::glam::{Quat, Vec3};

use crate::camera::movement_heading::MovementHeading;

/// Where the game's camera stands and how it is turned, in the engine's space: unturned, it looks along `+z` with `+y`
/// up, as X-Ray's camera does.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MovementCamera {
  pub position: Vec3,
  pub rotation: Quat,
}

impl Default for MovementCamera {
  fn default() -> Self {
    Self {
      position: Vec3::ZERO,
      rotation: Quat::IDENTITY,
    }
  }
}

impl MovementCamera {
  /// A camera at `position` facing `heading`.
  pub fn facing(position: Vec3, heading: MovementHeading) -> Self {
    Self {
      position,
      rotation: heading.get_rotation(),
    }
  }

  /// A camera at `position` looking at `target`, level: turned and pitched, never rolled. One looking at where it stands
  /// faces `+z`.
  pub fn looking_at(position: Vec3, target: Vec3) -> Self {
    Self::facing(position, MovementHeading::of(target - position))
  }

  /// Where it looks, a unit vector.
  pub fn get_forward(&self) -> Vec3 {
    self.rotation * Vec3::Z
  }

  /// Its up, a unit vector.
  pub fn get_up(&self) -> Vec3 {
    self.rotation * Vec3::Y
  }

  /// Its right, a unit vector.
  pub fn get_right(&self) -> Vec3 {
    self.rotation * Vec3::X
  }

  /// The way it faces, read back from its rotation.
  pub fn get_heading(&self) -> MovementHeading {
    MovementHeading::of(self.get_forward())
  }
}

impl LoopBlend for MovementCamera {
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    Self {
      position: self.position.lerp(next.position, alpha),
      rotation: self.rotation.slerp(next.rotation, alpha),
    }
  }
}
