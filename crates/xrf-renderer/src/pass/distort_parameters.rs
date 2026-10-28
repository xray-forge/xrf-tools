use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::distort_uniform::DistortUniform;

/// What the distortion before the resolve reads: the frame as it stood, what the water, the particles, the rain and
/// the distorting surfaces move it by and blur it by, the G-buffer's depth a move is refused nearer than, the engine's
/// bloom the blur goes towards, and how far it moves.
#[derive(PassParameters)]
#[parameters(group = 1)]
pub struct DistortParameters<'a> {
  #[texture(d2, unfilterable)]
  pub frame: GraphTexture,
  #[texture(d2, unfilterable)]
  pub distortion: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, float)]
  pub bloom_target: GraphTexture,
  #[sampler(filtering)]
  pub bloom_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub distort: UniformBinding<DistortUniform>,
}
