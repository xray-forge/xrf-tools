use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;

/// What the combine reads: the G-buffer and the light gathered over it, the material table, the frame's lighting and
/// exposure, the ambient occlusion and the haze map.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct CombineParameters<'a> {
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub light_target: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[texture(d2, unfilterable)]
  pub occlusion_target: GraphTexture,
  #[texture(d2, float)]
  pub haze_map: GraphTexture,
}
