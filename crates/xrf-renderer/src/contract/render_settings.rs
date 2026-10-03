use serde::{Deserialize, Serialize};

use crate::contract::render_frame_rate::RenderFrameRate;

/// What every viewport of the renderer draws with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
  pub frame_rate: RenderFrameRate,
  /// Whether each pass of a frame is timed on the GPU, where the device writes timestamps between passes.
  pub is_gpu_timed: bool,
}
