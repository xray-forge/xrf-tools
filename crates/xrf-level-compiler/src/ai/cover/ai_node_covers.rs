/// One node's covers in its four directions, from a standing eye and a crouching one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AiNodeCovers {
  pub(crate) high: [f32; 4],
  pub(crate) low: [f32; 4],
}
