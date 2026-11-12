use glam::Vec3;

/// A ray as the engine's collision queries take one: from `origin` along `direction`, out to `range`.
///
/// Distances along it are in multiples of `direction`'s length, metres for a unit direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
  pub origin: Vec3,
  pub direction: Vec3,
  pub range: f32,
}

impl Ray {
  pub const fn new(origin: Vec3, direction: Vec3, range: f32) -> Self {
    Self {
      origin,
      direction,
      range,
    }
  }

  /// Whether a crossing this far along counts, as `CDB::COLLIDER`'s ray query keeps one: ahead of the origin and not
  /// past the range.
  pub fn is_within(&self, distance: f32) -> bool {
    distance > 0.0 && distance <= self.range
  }
}
