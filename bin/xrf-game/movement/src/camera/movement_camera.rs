use xrf_game_loop::LoopBlend;
use xrf_math::glam::{EulerRot, Quat, Vec3};

/// Where the game's camera stands and how it is turned, in the engine's space: unturned, it looks along `+z` with `+y`
/// up, as X-Ray's camera does.
#[derive(Clone, Copy, Debug, PartialEq)]
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
  /// A camera at `position` looking at `target`, level: turned and pitched, never rolled. One looking at where it stands
  /// faces `+z`.
  pub fn looking_at(position: Vec3, target: Vec3) -> Self {
    let direction: Vec3 = (target - position).normalize_or(Vec3::Z);
    // Turning about `y` carries `+z` to `(sin yaw, 0, cos yaw)`; pitching about `x` lowers it by `sin pitch`.
    let yaw: f32 = direction.x.atan2(direction.z);
    let pitch: f32 = -direction.y.clamp(-1.0, 1.0).asin();

    Self {
      position,
      rotation: Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0),
    }
  }

  /// Where it looks, a unit vector.
  pub fn get_forward(&self) -> Vec3 {
    self.rotation * Vec3::Z
  }

  /// Its up, a unit vector.
  pub fn get_up(&self) -> Vec3 {
    self.rotation * Vec3::Y
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
