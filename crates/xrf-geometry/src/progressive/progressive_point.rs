use xrf_math::{Vector2d, Vector3d};

/// One vertex of a mesh made progressive: what a collapse is measured by, its place and its texture coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressivePoint {
  pub position: Vector3d,
  pub coordinate: Vector2d,
}

impl ProgressivePoint {
  /// The point in the quadric's five dimensions.
  pub(crate) fn to_quadric_point(self) -> [f64; 5] {
    [
      f64::from(self.position.x),
      f64::from(self.position.y),
      f64::from(self.position.z),
      f64::from(self.coordinate.x),
      f64::from(self.coordinate.y),
    ]
  }
}
