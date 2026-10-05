use xrf_renderer_core::{GraphTexture, PassParameters};

/// What FXAA reads: the scene as drawn, filtered.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct FxaaParameters<'a> {
  #[texture(d2, float)]
  pub frame: GraphTexture,
  #[sampler(filtering)]
  pub frame_sampler: &'a wgpu::Sampler,
}
