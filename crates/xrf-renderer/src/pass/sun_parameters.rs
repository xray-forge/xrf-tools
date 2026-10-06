use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::shadow_uniform::ShadowUniform;

/// What the sun's pass reads: the G-buffer's normals, material and depth, the material table, the frame's lighting, the
/// sun's shadow cascades and what they were drawn with, and the contact shadows under them.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SunParameters<'a> {
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[texture(d2_array, depth)]
  pub shadow_maps: GraphTexture,
  #[uniform]
  pub shadows: UniformBinding<ShadowUniform>,
  #[texture(d2, unfilterable)]
  pub contact_shadows: GraphTexture,
}
