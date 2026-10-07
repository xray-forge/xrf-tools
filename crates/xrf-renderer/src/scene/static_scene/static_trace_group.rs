use glam::Mat4;

use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;

/// An instanced mesh as the level's acceleration structures take it: its geometry once, and where each of its places
/// stands it, as a top-level instance's transform, those places running on from `first_place`.
#[derive(Clone, Debug)]
pub struct StaticTraceGroup {
  pub geometry: StaticTraceGeometry,
  pub first_place: u32,
  /// Each place's transform, three rows of the matrix's first three columns and its translation.
  pub transforms: Vec<[f32; 12]>,
}

impl StaticTraceGroup {
  /// A matrix as an instance's transform: rows of its linear part, each closed by the translation's coordinate.
  pub fn to_transform(matrix: &Mat4) -> [f32; 12] {
    let rows: [glam::Vec4; 3] = [matrix.row(0), matrix.row(1), matrix.row(2)];

    std::array::from_fn(|at| rows[at / 4][at % 4])
  }
}
