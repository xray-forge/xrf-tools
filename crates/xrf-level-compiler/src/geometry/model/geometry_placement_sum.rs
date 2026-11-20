/// The order one axis of `Fmatrix::transform_tiny` sums its three products and translation in, in single precision.
///
/// The source sums left to right, but SDK 0.7's xrLC binary reassociates them differently at each inlined call, and
/// its outputs are the reference, so each collision form names its calls' orders (`GeometryPlacement::RAY_MODEL_SUMS`
/// and `COLLISION_SUMS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GeometryPlacementSum {
  /// `((x + y) + z) + t`, the source's order.
  Ordered,
  /// `((y + z) + x) + t`.
  YzFirst,
  /// `((x + z) + y) + t`.
  XzFirst,
}

impl GeometryPlacementSum {
  /// The sum of the products `x`, `y` and `z` and `translation`, in this order.
  pub(crate) fn sum(self, x: f32, y: f32, z: f32, translation: f32) -> f32 {
    match self {
      Self::Ordered => x + y + z + translation,
      Self::YzFirst => (y + z) + x + translation,
      Self::XzFirst => (x + z) + y + translation,
    }
  }
}
