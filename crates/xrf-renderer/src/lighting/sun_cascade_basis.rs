use glam::Vec3;

/// The axes a cascade's square is laid on, as the engine takes them (`render_phase_sun.cpp`): `x` across the light
/// unless the light runs along it.
#[derive(Clone, Copy, Debug)]
pub struct SunCascadeBasis {
  /// Across the light.
  pub right: Vec3,
  /// Across the light and `right`.
  pub up: Vec3,
  /// Where the light travels, normalized.
  pub light: Vec3,
  /// The square's four sides' normals, pointing in: along `right`, against it, along `up` and against it.
  pub sides: [Vec3; 4],
}

impl SunCascadeBasis {
  /// The axes for light travelling along a direction; a zero one, which points nowhere, stands straight down, so the
  /// axes stay finite.
  pub fn new(direction: Vec3) -> Self {
    let light: Vec3 = direction.try_normalize().unwrap_or(Vec3::NEG_Y);
    let mut right: Vec3 = Vec3::X;

    if right.dot(light).abs() > 0.99 {
      right = Vec3::Z;
    }

    let up: Vec3 = light.cross(right).normalize();

    right = up.cross(light).normalize();

    Self {
      right,
      up,
      light,
      sides: [right, -right, up, -up],
    }
  }
}
