use crate::data::ogf_static_vertex::OgfStaticVertex;
use crate::data::ogf_vertex::OgfVertex;

/// Decoded vertices of a visual, in the family its kind reads them as: `CSkeletonX` for skinned children, `Fvisual`
/// for every other visual (`xray-16/src/Layers/xrRender/SkeletonX.cpp`, `FVisual.cpp`).
#[derive(Clone, Debug, PartialEq)]
pub enum OgfVertexData {
  /// `vertBoned1W` to `vertBoned4W`, each vertex linked to the bones it follows.
  Skinned(Vec<OgfVertex>),
  /// A flexible vertex format, `OGF_VERTICES` of a static visual.
  Static(Vec<OgfStaticVertex>),
}

impl OgfVertexData {
  /// Vertices held, whichever family they are.
  pub fn len(&self) -> usize {
    match self {
      Self::Skinned(vertices) => vertices.len(),
      Self::Static(vertices) => vertices.len(),
    }
  }

  /// Whether no vertex is held.
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}
