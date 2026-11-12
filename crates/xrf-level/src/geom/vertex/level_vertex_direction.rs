use xrf_math::Vector3d;

/// A vertex's `D3DCOLOR`-packed direction, decoded, and the byte its alpha carries beside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelVertexDirection {
  /// The direction, each component back in `-1..=1`.
  pub direction: Vector3d,
  /// The alpha byte: a normal's hemisphere term, or the low byte of a base coordinate for a tangent or binormal.
  pub extra: u8,
}
