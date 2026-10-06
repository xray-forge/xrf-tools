use serde::{Deserialize, Serialize};

use crate::contract::render_backend::RenderBackend;
use crate::contract::render_frame_rate::RenderFrameRate;
use crate::contract::render_graph_settings::RenderGraphSettings;

/// What every viewport of the renderer draws with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
  pub frame_rate: RenderFrameRate,
  /// Whether each pass of a frame is timed on the GPU, where the device writes timestamps between passes.
  pub is_gpu_timed: bool,
  /// Which of the frame graph's mechanisms the frames compile with.
  pub graph: RenderGraphSettings,
  /// The graphics API to draw with, none for the first that starts; one that cannot start falls back to the first that
  /// does, and `XRF_RENDER_BACKEND` overrides it.
  pub backend: Option<RenderBackend>,
}
