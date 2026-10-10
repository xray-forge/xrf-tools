use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::indirect_trace_uniform::IndirectTraceUniform;
use crate::pass::lighting_uniform::LightingUniform;

/// What the traced indirect light reads: the G-buffer's normals, depth and marks, the lighting and the material table a
/// hit is lit by, and its own noise, reach and intensity.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct IndirectTraceParameters<'a> {
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub indirect: UniformBinding<IndirectTraceUniform>,
}
