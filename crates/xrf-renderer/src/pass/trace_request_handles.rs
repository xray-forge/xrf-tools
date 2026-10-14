use xrf_renderer_core::{GraphBuffer, StorageArray, StorageArrayMut};

use crate::pass::trace_request::TraceRequest;
use crate::pass::trace_request_parameters::TraceRequestParameters;

/// A request list as a frame's graph imported it, emptied for the frame: what the pass filling it binds, and what the
/// scene pass reads it and dispatches by.
#[derive(Clone, Copy)]
pub struct TraceRequestHandles {
  pub requests: GraphBuffer,
  pub head: GraphBuffer,
  pub dispatch: GraphBuffer,
}

impl TraceRequestHandles {
  /// What the pass listing rays binds.
  pub fn to_listed(self) -> TraceRequestParameters {
    TraceRequestParameters {
      trace_requests: StorageArrayMut::new(self.requests),
      trace_request_head: StorageArrayMut::new(self.head),
    }
  }

  /// The requests as the scene pass reads them, and their head, whose wait it keeps.
  pub fn to_read(self) -> (StorageArray<TraceRequest>, StorageArrayMut<u32>) {
    (StorageArray::new(self.requests), StorageArrayMut::new(self.head))
  }
}
