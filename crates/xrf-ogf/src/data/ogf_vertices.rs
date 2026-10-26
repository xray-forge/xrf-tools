use crate::data::ogf_vertex_data::OgfVertexData;

/// Contents of an OGF vertex chunk: the stored format tag, the vertex count it declares, and the
/// decoded vertices when the layout of that format is known.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfVertices {
  pub format: u32,
  pub count: u32,
  pub data: Option<OgfVertexData>,
}

impl OgfVertices {
  /// Flatten the per-vertex bone links back into the order they were stored in, none for static vertices.
  pub fn collect_bone_indices(&self) -> Vec<u16> {
    match &self.data {
      Some(OgfVertexData::Skinned(vertices)) => vertices
        .iter()
        .flat_map(|vertex| vertex.links.iter().map(|link| link.bone))
        .collect(),
      _ => Vec::new(),
    }
  }
}
