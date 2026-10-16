use crate::pass::trace_parameters::TraceParameters;

/// How the bitmask search meets the frame's tracing: the scene's trace and the sky's group, where it is built, whether
/// DLSS ray reconstruction resolves the frame, and whether the indirect light is traced in the search's place.
#[derive(Clone, Copy)]
pub struct BitmaskTracing<'a> {
  pub traced: Option<(TraceParameters<'a>, &'a wgpu::BindGroup)>,
  pub is_reconstructed: bool,
  pub is_light_traced: bool,
}
