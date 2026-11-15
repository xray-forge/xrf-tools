use xrf_math::{Vector2d, Vector3d};

/// One corner of a LOD's impostor quad (`OGF_LOD::_vertex`): its place in the level, its coordinate in the LOD
/// texture, and its light as the model's vertices around it carry it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeometryLodVertex {
  pub position: Vector3d,
  pub texture_coordinate: Vector2d,
  /// Red, green, blue and hemisphere, a byte each.
  pub light: [u8; 4],
  pub sun: u8,
}
