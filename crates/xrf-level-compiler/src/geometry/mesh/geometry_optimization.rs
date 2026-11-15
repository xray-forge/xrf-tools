/// What `GeometryMesh::optimize` removed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GeometryOptimization {
  /// Vertices welded onto another.
  pub(crate) welded_vertices: usize,
  /// Faces dropped as two of their corners welded together.
  pub(crate) degenerate_faces: usize,
  /// Vertices dropped as no face used them.
  pub(crate) isolated_vertices: usize,
}
