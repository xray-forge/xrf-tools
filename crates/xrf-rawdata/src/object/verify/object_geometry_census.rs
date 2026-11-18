use serde::Serialize;

/// What the geometry comparison counted.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectGeometryCensus {
  pub children: usize,
  pub reference_children: usize,
  pub vertices: usize,
  /// Compiled vertices one of the reference's stands in for: same position, normal, texture coordinate and weights.
  pub vertices_matched: usize,
  /// Compiled vertices that would match but for their normal: drawn and moved alike, shaded otherwise, as smoothing
  /// read by another rule gives.
  pub vertices_normal_only: usize,
  /// Distinct faces each side draws, a face written twice counted once and its mirror apart.
  pub faces: usize,
  pub reference_faces: usize,
  /// Reference faces the compile draws too, by their corners' positions in winding order.
  pub faces_matched: usize,
  /// Reference faces with two corners at one position, which cover nothing and which a compile drops.
  pub reference_flat_faces: usize,
}
