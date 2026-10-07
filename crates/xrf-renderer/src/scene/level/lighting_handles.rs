use xrf_renderer_core::{GraphBuffer, GraphTexture, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::shadow_uniform::ShadowUniform;

/// What a view's lit passes share in a frame's graph: the lighting and the sun's shadow uniforms as pushed, the
/// exposure's state, the material table and the sun's shadow maps as imported, and the table's sampler.
#[derive(Clone, Copy)]
pub struct LightingHandles<'a> {
  pub lighting: UniformBinding<LightingUniform>,
  pub exposure: GraphBuffer,
  pub material_lut: GraphTexture,
  pub lut_sampler: &'a wgpu::Sampler,
  pub shadow_maps: GraphTexture,
  pub shadows: UniformBinding<ShadowUniform>,
}
