use crate::feature::lighting::lights::tracing::light_tracing_parameters::LightTracingParameters;
use crate::feature::tracing::view_trace::trace_parameters::TraceParameters;

/// What tracing the lights' visibility binds besides the lights' own parameters: the tracing's targets, the level's
/// geometry, and the texture cache's group.
#[derive(Clone, Copy)]
pub(crate) struct LightTraceBindings<'a> {
  pub tracing: &'a LightTracingParameters,
  pub trace: &'a TraceParameters<'a>,
  pub textures: &'a wgpu::BindGroup,
}
