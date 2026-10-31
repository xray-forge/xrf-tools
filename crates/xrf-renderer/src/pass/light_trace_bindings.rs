use crate::pass::light_tracing_parameters::LightTracingParameters;
use crate::pass::trace_parameters::TraceParameters;

/// What tracing the lights' visibility binds besides the lights' own parameters: the tracing's targets, the level's
/// geometry, and the texture cache's group.
#[derive(Clone, Copy)]
pub struct LightTraceBindings<'a> {
  pub tracing: &'a LightTracingParameters,
  pub trace: &'a TraceParameters<'a>,
  pub textures: &'a wgpu::BindGroup,
}
