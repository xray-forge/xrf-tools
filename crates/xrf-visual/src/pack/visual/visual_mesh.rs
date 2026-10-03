/// One surface's triangles built rather than read: in renderer space, counter-clockwise from outside, as a packed
/// visual stores them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisualMesh {
  pub texture_name: Option<String>,
  pub shader_name: Option<String>,
  pub positions: Vec<[f32; 3]>,
  /// One a vertex, unit length.
  pub normals: Vec<[f32; 3]>,
  pub uvs: Vec<[f32; 2]>,
  pub indices: Vec<u16>,
}
