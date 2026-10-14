use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::sun_trace_uniform::SunTraceUniform;

/// What the sun's traced shadow reads: the G-buffer's normals, material and depth, and the sun it traces towards.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SunTraceParameters {
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[uniform]
  pub sun_trace: UniformBinding<SunTraceUniform>,
}
