use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;

/// What the enhanced drops and splashes read beside the rain: the scene blurred at an eighth of its size, which they
/// bend, the drop's and the splash's maps and their sampler, the depth the bend is checked against, the light their
/// glint is taken from, and the lighting and exposure the light they scatter is shown at.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct RainLookParameters<'a> {
  #[texture(d2, float)]
  pub refracted: GraphTexture,
  #[sampler(filtering)]
  pub refracted_sampler: &'a wgpu::Sampler,
  #[texture(d2, float)]
  pub drop_map: GraphTexture,
  #[texture(d2, float)]
  pub splash_map: GraphTexture,
  #[sampler(filtering)]
  pub map_sampler: &'a wgpu::Sampler,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, float)]
  pub light_target: GraphTexture,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
}
