use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::wet_uniform::WetUniform;

/// What the wet patches read: the G-buffer's depth, albedo and normals, the rain's cover, the splash volume and the
/// flow, and the wet surfaces' settings.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct WetPatchParameters<'a> {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, depth)]
  pub cover: GraphTexture,
  #[texture(d2_array, float)]
  pub splash: GraphTexture,
  #[texture(d2, float)]
  pub flow: GraphTexture,
  #[sampler(filtering)]
  pub wet_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub wet: UniformBinding<WetUniform>,
}
