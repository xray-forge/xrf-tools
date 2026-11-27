use std::f32::consts::FRAC_PI_2;

use serde::{Deserialize, Serialize};
use xrf_math::glam::{EulerRot, Quat, Vec3};

/// Which way a camera faces, as a turn and a pitch with no roll, in the engine's space: yaw turns from `+z` towards `+x`,
/// pitch lowers the view, both in radians.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MovementHeading {
  pub yaw: f32,
  pub pitch: f32,
}

impl MovementHeading {
  /// Just short of straight up or down, so the view never flips over the horizon.
  pub const MOST_PITCH: f32 = FRAC_PI_2 - 0.001;

  /// The heading of a direction; none given faces `+z`.
  pub fn of(direction: Vec3) -> Self {
    let direction: Vec3 = direction.normalize_or(Vec3::Z);

    // Turning about `y` carries `+z` to `(sin yaw, 0, cos yaw)`; pitching about `x` lowers it by `sin pitch`.
    Self {
      yaw: direction.x.atan2(direction.z),
      pitch: -direction.y.clamp(-1.0, 1.0).asin(),
    }
  }

  /// This heading turned by `yaw` and pitched by `pitch`, the pitch held short of straight up or down.
  pub fn turned(self, yaw: f32, pitch: f32) -> Self {
    Self {
      yaw: self.yaw + yaw,
      pitch: (self.pitch + pitch).clamp(-Self::MOST_PITCH, Self::MOST_PITCH),
    }
  }

  pub fn get_rotation(self) -> Quat {
    Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
  }
}
