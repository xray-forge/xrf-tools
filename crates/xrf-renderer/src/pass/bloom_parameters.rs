use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::bloom_uniform::BloomUniform;

/// What one of the bloom's draws reads: the high target for the build, else the bloom target it blurs, and its uniform.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct BloomParameters<'a> {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[sampler(filtering)]
  pub source_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub bloom: UniformBinding<BloomUniform>,
}
