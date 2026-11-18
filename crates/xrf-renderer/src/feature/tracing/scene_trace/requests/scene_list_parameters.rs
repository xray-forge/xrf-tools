use xrf_renderer_core::{PassParameters, StorageArray, StorageArrayMut};

use crate::feature::tracing::scene_trace::requests::trace_request::TraceRequest;

/// The scene pass's way in from a request list (`cs_scene`): the rays and their head, whose wait the scene pass keeps.
/// Group 5, empty for the water's every-pixel rays.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 5)]
pub(crate) struct SceneListParameters {
  #[storage]
  pub trace_requests: StorageArray<TraceRequest>,
  #[storage]
  pub trace_request_head: StorageArrayMut<u32>,
}
