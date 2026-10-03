use serde::{Deserialize, Serialize};

use crate::contract::render_presentation::RenderPresentation;

/// What every viewport of the renderer draws with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
  pub presentation: RenderPresentation,
  /// Whether each pass of a frame is timed on the GPU, where the device writes timestamps between passes.
  pub is_gpu_timed: bool,
}
