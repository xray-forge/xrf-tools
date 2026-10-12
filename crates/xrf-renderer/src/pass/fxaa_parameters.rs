use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::smoothing_uniform::SmoothingUniform;

/// What FXAA reads: the scene as drawn, filtered.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct FxaaParameters<'a> {
  #[texture(d2, float)]
  pub frame: GraphTexture,
  #[uniform]
  pub smoothing: UniformBinding<SmoothingUniform>,
  #[sampler(filtering)]
  pub frame_sampler: &'a wgpu::Sampler,
}
