use xrf_renderer_core::{PassParameters, StorageArray, StorageArrayMut};

use crate::feature::tracing::scene_trace::requests::trace_request::TraceRequest;

/// The scene pass's way in from a list (`cs_scene`): the rays a pass left in it, and their head, whose wait the scene
/// pass keeps. Group 5, which the water's every-pixel rays leave empty.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 5)]
pub(crate) struct SceneListParameters {
  #[storage]
  pub trace_requests: StorageArray<TraceRequest>,
  #[storage]
  pub trace_request_head: StorageArrayMut<u32>,
}
