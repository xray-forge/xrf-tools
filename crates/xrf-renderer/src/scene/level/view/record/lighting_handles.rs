use xrf_renderer_core::{GraphBuffer, GraphTexture, UniformBinding};

use crate::feature::lighting::shadows::shadow_uniform::ShadowUniform;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// The frame-graph resources a view's lit passes share: lighting and sun shadow uniforms, exposure, material and BRDF
/// tables, sun shadow maps, and their samplers.
#[derive(Clone, Copy)]
pub(crate) struct LightingHandles<'a> {
  pub lighting: UniformBinding<LightingUniform>,
  pub exposure: GraphBuffer,
  pub material_lut: GraphTexture,
  pub brdf_table: GraphTexture,
  pub lut_sampler: &'a wgpu::Sampler,
  pub shadow_maps: GraphTexture,
  pub shadows: UniformBinding<ShadowUniform>,
  pub shadow_compare: &'a wgpu::Sampler,
}
