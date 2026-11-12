use serde::{Deserialize, Serialize};
use xrf_math::Vector3d;

/// The two vectors spanning a domain's plane or crossing its axis, the engine's `pDomain::u` and `v`.
///
/// Serialized as a two-element array, the shape the domain's JSON carries.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(from = "(Vector3d, Vector3d)", into = "(Vector3d, Vector3d)")]
pub struct ParticleDomainBasis {
  /// The first: a triangle's or rectangle's first edge, or a unit vector across a disc's or cylinder's axis.
  pub u: Vector3d,
  /// The second: the other edge, or the unit vector across the axis and `u`.
  pub v: Vector3d,
}

impl From<(Vector3d, Vector3d)> for ParticleDomainBasis {
  fn from((u, v): (Vector3d, Vector3d)) -> Self {
    Self { u, v }
  }
}

impl From<ParticleDomainBasis> for (Vector3d, Vector3d) {
  fn from(basis: ParticleDomainBasis) -> Self {
    (basis.u, basis.v)
  }
}
