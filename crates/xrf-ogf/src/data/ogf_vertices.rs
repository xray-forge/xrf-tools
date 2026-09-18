use crate::data::ogf_vertex::OgfVertex;

/// Contents of an OGF vertex chunk: the stored format tag, the vertex count it declares, and the
/// decoded vertices when the layout of that format is known.
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
