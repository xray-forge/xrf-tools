use crate::pass::trace_request_handles::TraceRequestHandles;

/// A list of rays the scene pass traces, and the labels its two passes go by: the dispatch copied out of its head, then
/// the trace.
#[derive(Clone, Copy)]
pub struct SceneTraceRequest {
  pub dispatch_label: &'static str,
  pub trace_label: &'static str,
  pub handles: TraceRequestHandles,
}
