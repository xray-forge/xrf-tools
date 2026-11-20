use glam::Vec3A;

/// A ray as a box test takes it: its origin and the inverse of its direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SlabRay {
  pub(crate) origin: Vec3A,
  /// Each component's inverse, a zero component of either sign as plus infinity.
  pub(crate) inverse: Vec3A,
}

impl SlabRay {
  pub(crate) fn new(origin: Vec3A, direction: Vec3A) -> Self {
    Self {
      origin,
      // Adding zero turns a minus zero into plus zero, whose sign the infinite inverse then takes.
      inverse: direction.recip().abs().copysign(direction + Vec3A::ZERO),
    }
  }

  /// Whether the ray reaches the box from `low` to `high` within `reach` of its origin, without branching on its
  /// direction.
  ///
  /// A ray parallel to an axis meets the box's faces there at infinity. An origin exactly on such a face gives a NaN,
  /// taken as inside: left to `min` and `max`, which drop a NaN, the ray running along the face would miss the box.
  pub(crate) fn reaches(&self, reach: f32, low: Vec3A, high: Vec3A) -> bool {
    let first: Vec3A = (low - self.origin) * self.inverse;
    let second: Vec3A = (high - self.origin) * self.inverse;
    let first: Vec3A = Vec3A::select(first.is_nan_mask(), Vec3A::NEG_INFINITY, first);
    let second: Vec3A = Vec3A::select(second.is_nan_mask(), Vec3A::INFINITY, second);
    let near: f32 = first.min(second).max_element().max(0.0);
    let far: f32 = first.max(second).min_element().min(reach);

    near <= far
  }
}
