use xrf_math::{Vector2d, Vector3d};

use crate::geometry::surface::geometry_basis::GeometryBasis;
use crate::light::point::light_color::LightColor;

/// One vertex of a visual before it is quantized, `OGF_Vertex`. A weld keeps the first corner's light, as
/// `OGF_Vertex::similar` does not compare it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GeometryVisualVertex {
  pub position: Vector3d,
  pub normal: Vector3d,
  pub basis: GeometryBasis,
  pub texture_coordinate: Vector2d,
  /// The lightmap coordinate, for a visual lit from a lightmap or implicitly; none for a vertex-lit one.
  pub lightmap_coordinate: Option<Vector2d>,
  /// Its light, `OGF_Vertex::Color`: the sky term for every vertex, the static light and sun too for a vertex-lit one.
  pub light: LightColor,
}
