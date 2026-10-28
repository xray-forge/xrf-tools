use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::distort_uniform::DistortUniform;

/// What the distortion before the resolve reads: the frame as it stood, what the water, the particles, the rain and
/// the distorting surfaces move it by, the G-buffer's depth a move is refused nearer than, and how far it moves.
#[derive(PassParameters)]
#[parameters(group = 1)]
pub struct DistortParameters {
  #[texture(d2, unfilterable)]
  pub frame: GraphTexture,
  #[texture(d2, unfilterable)]
  pub distortion: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[uniform]
  pub distort: UniformBinding<DistortUniform>,
}
