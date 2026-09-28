use crate::geom::vertex::level_vertex_bytes::LevelVertexBytes;
use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;

/// One visual's vertices as the level stores them, beside the declaration saying where each attribute sits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelVertexPayload {
  layout: LevelVertexLayout,
  /// A whole number of strides, which is what lets each vertex be read through its layout unchecked.
  bytes: Vec<u8>,
}

impl LevelVertexPayload {
  /// A payload of whole vertices, which the source reading it guarantees.
  pub(crate) fn new(layout: LevelVertexLayout, bytes: Vec<u8>) -> Self {
    debug_assert!(
      bytes.len().is_multiple_of(layout.get_stride() as usize),
      "a payload holds whole vertices"
    );

    Self { layout, bytes }
  }

  /// The declaration the vertices were stored in.
  pub const fn get_layout(&self) -> &LevelVertexLayout {
    &self.layout
  }

  /// The vertices' bytes as stored, a stride each.
  pub fn get_bytes(&self) -> &[u8] {
    &self.bytes
  }

  /// Each vertex, a stride of bytes read through the layout.
  pub fn vertices(&self) -> impl Iterator<Item = LevelVertexBytes<'_>> {
    self
      .bytes
      .chunks_exact(self.layout.get_stride() as usize)
      .map(|bytes| LevelVertexBytes::new(&self.layout, bytes))
  }

  /// How many vertices it holds.
  pub fn len(&self) -> usize {
    self.bytes.len() / self.layout.get_stride() as usize
  }

  /// Whether it holds none.
  pub fn is_empty(&self) -> bool {
    self.bytes.is_empty()
  }
}
