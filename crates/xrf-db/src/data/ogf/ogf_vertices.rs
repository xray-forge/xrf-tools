use crate::data::ogf::ogf_vertex::OgfVertex;

/// Contents of an OGF vertex chunk: the stored format tag, the vertex count it declares, and the
/// decoded vertices when the layout of that format is known.
///
/// Kept separate from [`crate::OgfGeometry`] because a vertex chunk is meaningful on its own, while
/// geometry is the pairing of vertices with indices. The count is retained alongside the vertices
/// rather than derived from them, so a file whose declared count disagrees with its payload can still
/// be described rather than rejected.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfVertices {
  pub format: u32,
  pub count: u32,
  pub vertices: Option<Vec<OgfVertex>>,
}

impl OgfVertices {
  /// Flatten the per-vertex bone links back into the order they were stored in.
  pub fn collect_bone_indices(&self) -> Vec<u16> {
    self
      .vertices
      .as_ref()
      .map(|vertices| {
        vertices
          .iter()
          .flat_map(|vertex| vertex.links.iter().map(|link| link.bone))
          .collect()
      })
      .unwrap_or_default()
  }
}
