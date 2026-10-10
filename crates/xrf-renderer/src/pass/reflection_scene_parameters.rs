use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, StorageArrayMut, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::trace_request::TraceRequest;

/// What the reflections' scene pass reads and writes: the lighting and the material table a hit is lit by, and the
/// exposure a water's is tonemapped at; the rays a pass left to it and their head, whose wait it keeps; and the target
/// and the ends into which each ray that meets the level's geometry writes its answer, the surfaces' traced reflection
/// and its ends or the water's reflection and no ends.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct ReflectionSceneParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[storage]
  pub trace_requests: StorageArray<TraceRequest>,
  #[storage]
  pub trace_request_head: StorageArrayMut<u32>,
  #[storage_texture(d2, rgba16float, write)]
  pub traced_out: GraphTexture,
  #[storage_texture(d2, r32float, write)]
  pub ends_out: GraphTexture,
}
