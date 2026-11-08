use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::feature::image::exposure::reading::exposure_head::ExposureHead;
use crate::feature::lighting::shadows::shadow_uniform::ShadowUniform;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// What the composited surfaces read beside their textures: the frame's lighting and exposure, the material table, and
/// the sun's shadow cascades.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub(crate) struct CompositedParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[texture(d2_array, depth)]
  pub shadow_maps: GraphTexture,
  #[uniform]
  pub shadows: UniformBinding<ShadowUniform>,
}
