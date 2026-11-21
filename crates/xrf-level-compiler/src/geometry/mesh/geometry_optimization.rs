/// What `GeometryMesh::optimize` removed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GeometryOptimization {
  /// Vertices welded onto another.
  pub welded_vertices: usize,
  /// Faces dropped as two of their corners welded together.
  pub degenerate_faces: usize,
  /// Vertices dropped as no face used them.
  pub isolated_vertices: usize,
}
