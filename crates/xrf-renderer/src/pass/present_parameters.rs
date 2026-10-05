use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::present_uniform::PresentUniform;

/// What the present pass reads of a viewport's frame: its finished scene and the targets a debug view shows, what the
/// water and the particles distort it by, the frame upscaled or the scene again, its bloom, and what it shows.
#[derive(PassParameters)]
#[parameters(group = 1)]
pub struct PresentParameters<'a> {
  #[texture(d2, unfilterable)]
  pub scene: GraphTexture,
  #[texture(d2, unfilterable)]
  pub distortion: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub light_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub occlusion_target: GraphTexture,
  #[uniform]
  pub present: UniformBinding<PresentUniform>,
  #[texture(d2, unfilterable)]
  pub upscaled: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[texture(d2, float)]
  pub bloom_target: GraphTexture,
  #[sampler(filtering)]
  pub bloom_sampler: &'a wgpu::Sampler,
}
