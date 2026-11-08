use crate::feature::tracing::reflections::trace::reflection_trace_out_parameters::ReflectionTraceOutParameters;
use crate::feature::tracing::scene_trace::requests::trace_request_parameters::TraceRequestParameters;

/// What the reflections' budgeted trace writes: the list of rays it leaves to the scene pass, and its targets.
#[derive(Clone, Copy)]
pub(crate) struct BudgetedTraceOutputs<'a> {
  pub requests: &'a TraceRequestParameters,
  pub out: &'a ReflectionTraceOutParameters,
}
