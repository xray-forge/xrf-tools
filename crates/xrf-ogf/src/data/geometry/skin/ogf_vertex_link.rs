/// One bone a vertex is skinned to, and how strongly.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OgfVertexLink {
  pub bone: u16,
  pub weight: f32,
}
