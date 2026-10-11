use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::pass::trace_request::TraceRequest;

/// What the reflections' scene pass reads and writes: the lighting, the settings and the material table a hit is lit
/// by; the rays the trace left to it and their head; and the traced reflection and its ends, into which each ray
/// that meets the level's geometry writes its answer.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct ReflectionSceneParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub reflection: UniformBinding<ReflectionUniform>,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[storage]
  pub trace_requests: StorageArray<TraceRequest>,
  #[storage]
  pub trace_request_head: StorageArray<u32>,
  #[storage_texture(d2, rgba16float, write)]
  pub traced_out: GraphTexture,
  #[storage_texture(d2, r32float, write)]
  pub ends_out: GraphTexture,
}
