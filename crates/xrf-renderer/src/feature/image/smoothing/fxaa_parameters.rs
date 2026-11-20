use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::feature::image::smoothing::smoothing_uniform::SmoothingUniform;

/// What FXAA reads: the scene as drawn, filtered.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub(crate) struct FxaaParameters<'a> {
  #[texture(d2, float)]
  pub frame: GraphTexture,
  #[uniform]
  pub smoothing: UniformBinding<SmoothingUniform>,
  #[sampler(filtering)]
  pub frame_sampler: &'a wgpu::Sampler,
}
