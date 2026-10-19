use xrf_renderer_core::{GraphTexture, PassParameters};

/// What the enhanced rain's blur of the scene reads: the scene and the sampler it is filtered through.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct RainBlurParameters<'a> {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[sampler(filtering)]
  pub source_sampler: &'a wgpu::Sampler,
}
