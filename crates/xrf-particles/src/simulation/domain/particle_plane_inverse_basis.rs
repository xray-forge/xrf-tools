use glam::Vec3;

/// The rows that read a point's coordinates along a rectangle's or triangle's `u` and `v` edges, the engine's `s1` and
/// `s2`: a point's offset from the corner dotted with each gives its coordinate along that edge, in edge lengths.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ParticlePlaneInverseBasis {
  /// The row reading the coordinate along `u`.
  pub s1: Vec3,
  /// The row reading the coordinate along `v`.
  pub s2: Vec3,
}
