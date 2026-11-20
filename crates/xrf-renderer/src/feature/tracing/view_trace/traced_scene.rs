use crate::feature::tracing::view_trace::trace_parameters::TraceParameters;

/// The level's geometry as a frame traces rays against it: the trace's parameters every consumer binds, and the
/// texture cache's group a hit samples.
#[derive(Clone, Copy)]
pub(crate) struct TracedScene<'a> {
  pub trace: TraceParameters<'a>,
  pub textures: &'a wgpu::BindGroup,
}
