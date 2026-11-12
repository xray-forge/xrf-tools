use serde::{Deserialize, Serialize};
use xrf_math::Vector3d;

/// The two vectors a domain places itself by, the engine's `pDomain::p1` and `p2`, read by shape.
///
/// Serialized as a two-element array, the shape the domain's JSON carries.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(from = "(Vector3d, Vector3d)", into = "(Vector3d, Vector3d)")]
pub struct ParticleDomainPoints {
  /// The domain's point: its corner, centre, start, or a point on its plane.
  pub p1: Vector3d,
  /// The domain's second vector: a box's far corner, the way to a line's or axis's end, or a plane's normal.
  pub p2: Vector3d,
}

impl From<(Vector3d, Vector3d)> for ParticleDomainPoints {
  fn from((p1, p2): (Vector3d, Vector3d)) -> Self {
    Self { p1, p2 }
  }
}

impl From<ParticleDomainPoints> for (Vector3d, Vector3d) {
  fn from(points: ParticleDomainPoints) -> Self {
    (points.p1, points.p2)
  }
}
