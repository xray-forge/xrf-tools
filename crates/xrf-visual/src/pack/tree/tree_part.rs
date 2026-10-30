/// One mesh a tree type draws, dressed by one shader: the first visual of the level naming it, and its triangles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreePart {
  pub visual: u32,
  pub shader_id: u16,
  pub triangles: u32,
}
