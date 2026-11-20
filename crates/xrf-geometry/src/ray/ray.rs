use glam::{Vec3, Vec3A};

/// A ray as the engine's collision queries take one: from `origin` along `direction`, out to `range`. Its vectors are
/// held a SIMD register wide, as [`crate::Triangle`]'s corners are.
///
/// Distances along it are in multiples of `direction`'s length, metres for a unit direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
  pub origin: Vec3A,
  pub direction: Vec3A,
  pub range: f32,
}

impl Ray {
  pub fn new(origin: Vec3, direction: Vec3, range: f32) -> Self {
    Self {
      origin: Vec3A::from(origin),
      direction: Vec3A::from(direction),
      range,
    }
  }

  /// Whether a crossing this far along counts, as `CDB::COLLIDER`'s ray query keeps one: ahead of the origin and not
  /// past the range.
  pub fn is_within(&self, distance: f32) -> bool {
    distance > 0.0 && distance <= self.range
  }
}
