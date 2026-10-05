use xrf_renderer_core::{GraphTexture, PassParameters};

/// What a stage of SMAA reads: the scene as drawn, the edges and the blend weights (an empty texture where the stage
/// writes them or comes before them), the area and search lookups, and its two samplers.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct SmaaParameters<'a> {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[texture(d2, float)]
  pub edges_texture: GraphTexture,
  #[texture(d2, float)]
  pub weights_texture: GraphTexture,
  #[texture(d2, float)]
  pub area_texture: GraphTexture,
  #[texture(d2, float)]
  pub search_texture: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
  #[sampler(filtering)]
  pub point_sampler: &'a wgpu::Sampler,
}
