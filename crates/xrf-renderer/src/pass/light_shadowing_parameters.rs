use xrf_renderer_core::{GraphTexture, PassParameters};

/// What the light visibility pass shadows the lights from and writes: the scene's light shadow atlas and the sampler
/// comparing it, and the visibility target, a share a slot, four slots a layer.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub struct LightShadowingParameters<'a> {
  #[texture(d2, depth)]
  pub shadow_atlas: GraphTexture,
  #[sampler(comparison)]
  pub shadow_sampler: &'a wgpu::Sampler,
  #[storage_texture(d2_array, rgba8unorm, write)]
  pub light_visibility_out: GraphTexture,
}
