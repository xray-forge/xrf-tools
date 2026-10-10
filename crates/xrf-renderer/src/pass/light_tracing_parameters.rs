use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::light_trace_uniform::LightTraceUniform;

/// What the lamps' traced visibility pass reads and writes: how it traces, and the visibility target, a share a slot,
/// four slots a layer.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct LightTracingParameters {
  #[uniform]
  pub light_trace: UniformBinding<LightTraceUniform>,
  #[storage_texture(d2_array, rgba8unorm, write)]
  pub light_visibility_out: GraphTexture,
}
