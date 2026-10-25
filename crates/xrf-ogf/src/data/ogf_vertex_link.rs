/// One bone a vertex is skinned to, and how strongly.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfVertexLink {
  pub bone: u16,
  pub weight: f32,
}
