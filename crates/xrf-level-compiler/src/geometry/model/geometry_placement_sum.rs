/// The order one axis of `Fmatrix::transform_tiny` sums its terms in: the point's three coordinates, each by its
/// matrix entry, and the translation, in single precision.
///
/// The source sums them left to right, but SDK 0.7's xrLC was compiled to reassociate the sums, and in a different
/// order at each of its inlined calls, so a corner placed in its collision forms carries the order of the call that
/// placed it. Its outputs are the reference, so each form names the orders of its own calls (`GeometryPlacement`'s
/// `RAY_MODEL_SUMS` and `COLLISION_SUMS`).
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
  /// The sum of `x`, `y` and `z`, the coordinates' products, and `translation`, in this order.
  pub(crate) fn sum(self, x: f32, y: f32, z: f32, translation: f32) -> f32 {
    match self {
      Self::Ordered => x + y + z + translation,
      Self::YzFirst => (y + z) + x + translation,
      Self::XzFirst => (x + z) + y + translation,
    }
  }
}
