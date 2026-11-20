use xrf_renderer_core::{PassParameters, ShaderAtomicU32, StorageArray, StorageArrayMut};

use crate::feature::tracing::scene_trace::requests::trace_request::TraceRequest;

/// Where the reflections' trace lists the rays it leaves to the scene pass: the requests, and their head, the count
/// made and the scene pass's dispatch, both counted up as requests are made; and the wait the head held as the frame
/// started, read plainly.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub(crate) struct TraceRequestParameters {
  #[storage]
  pub trace_requests: StorageArrayMut<TraceRequest>,
  #[storage]
  pub trace_request_head: StorageArrayMut<ShaderAtomicU32>,
  #[storage]
  pub trace_request_waits: StorageArray<u32>,
}
