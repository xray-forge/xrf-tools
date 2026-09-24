use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;

/// One visual's vertices as the level stores them, beside the declaration saying where each attribute sits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelVertexPayload {
  pub layout: LevelVertexLayout,
  pub bytes: Vec<u8>,
}

impl LevelVertexPayload {
  /// Each vertex's bytes, a stride of them.
  pub fn vertices(&self) -> impl Iterator<Item = &[u8]> {
    self.bytes.chunks_exact(self.layout.stride as usize)
  }

  /// How many vertices it holds.
  pub fn len(&self) -> usize {
    self.bytes.len() / self.layout.stride as usize
  }

  /// Whether it holds none.
  pub fn is_empty(&self) -> bool {
    self.bytes.is_empty()
  }
}
