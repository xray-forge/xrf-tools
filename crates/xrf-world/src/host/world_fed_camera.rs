use xrf_math::glam::Vec3;

/// A camera its host stands, in the engine's space, and its lens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldFedCamera {
  pub position: Vec3,
  /// Where it looks and its up, unit vectors.
  pub forward: Vec3,
  pub up: Vec3,
  /// Vertical field of view in degrees, and the near and far planes in metres.
  pub field_of_view: f32,
  pub near: f32,
  pub far: f32,
}

impl Default for WorldFedCamera {
  /// At the origin looking along the engine's `+z`, through the world's own fly camera's lens.
  fn default() -> Self {
    Self {
      position: Vec3::ZERO,
      forward: Vec3::Z,
      up: Vec3::Y,
      field_of_view: 67.5,
      near: 0.2,
      far: 10_000.0,
    }
  }
}
