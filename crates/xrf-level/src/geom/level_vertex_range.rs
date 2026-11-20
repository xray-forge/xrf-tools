use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;

/// Where one visual's vertices sit in their buffer's payload, and the declaration they were stored in.
pub(crate) struct LevelVertexRange {
  pub(crate) layout: LevelVertexLayout,
  pub(crate) offset: u64,
  pub(crate) size: usize,
}
