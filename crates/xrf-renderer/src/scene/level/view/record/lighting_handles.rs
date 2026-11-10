use xrf_renderer_core::{GraphBuffer, GraphTexture, UniformBinding};

use crate::feature::lighting::shadows::shadow_uniform::ShadowUniform;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// What a view's lit passes share in a frame's graph: the lighting and the sun's shadow uniforms as pushed, the
/// exposure's state, the material table, the BRDF's split-sum table and the sun's shadow maps as imported, the
/// material table's sampler, and the comparison the sun's maps are filtered through.
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
