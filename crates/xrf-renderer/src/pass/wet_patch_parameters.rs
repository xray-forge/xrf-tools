use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::wet_uniform::WetUniform;

/// What the wet patches read: the G-buffer's depth, albedo, normals and material marks, the rain's cover, the level's
/// surface's lowest heights, its puddle sites and its water, the splash volume and the flow, the enhanced wetting's ripples, puddle noise and puddle normals, and the wet surfaces' settings.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct WetPatchParameters<'a> {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub cover: GraphTexture,
  #[texture(d2, unfilterable)]
  pub surface_lowest: GraphTexture,
  #[texture(d2, unfilterable)]
  pub puddle_sites: GraphTexture,
  #[texture(d2, depth)]
  pub surface_water: GraphTexture,
  #[texture(d2_array, float)]
  pub splash: GraphTexture,
  #[texture(d2, float)]
  pub flow: GraphTexture,
  #[texture(d2, float)]
  pub ripples: GraphTexture,
  #[texture(d2, float)]
  pub puddle_noise: GraphTexture,
  #[texture(d2, float)]
  pub puddle_normal: GraphTexture,
  #[sampler(filtering)]
  pub wet_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub wet: UniformBinding<WetUniform>,
}
