use xrf_math::{Vector2d, Vector3d};

use crate::geometry::geometry_basis::GeometryBasis;

/// One vertex of a visual as xrLC welds it, `OGF_Vertex`: what each face corner carries into its visual, before it is
/// lit and quantized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeometryVisualVertex {
  pub position: Vector3d,
  pub normal: Vector3d,
  pub basis: GeometryBasis,
  pub texture_coordinate: Vector2d,
  /// The lightmap coordinate, for a visual lit from a lightmap or implicitly; none for a vertex-lit one.
  pub lightmap_coordinate: Option<Vector2d>,
}
