use crate::pass::reflection_trace_out_parameters::ReflectionTraceOutParameters;
use crate::pass::trace_request_parameters::TraceRequestParameters;

/// What the reflections' budgeted trace writes: the list of rays it leaves to the scene pass, and its targets.
#[derive(Clone, Copy)]
pub struct BudgetedTraceOutputs<'a> {
  pub requests: &'a TraceRequestParameters,
  pub out: &'a ReflectionTraceOutParameters,
}
