use serde::{Deserialize, Serialize};

use crate::contract::render_backend::RenderBackend;
use crate::contract::render_frame_rate::RenderFrameRate;
use crate::contract::render_gpu::RenderGpu;
use crate::contract::render_graph_settings::RenderGraphSettings;
use crate::frame::periodic_scheduler::PeriodicScheduler;

/// GPU milliseconds a frame gives the work drawn once in so many frames that may wait, unless the settings say
/// otherwise.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_DEFAULT_PERIODIC_BUDGET: f32 = PeriodicScheduler::DEFAULT_BUDGET;

/// What every viewport of the renderer draws with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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
  /// The GPU to draw on, none for the one the backend picks automatically (its high-performance adapter); one the
  /// backend does not have falls back to automatic.
  pub gpu: Option<RenderGpu>,
  /// Milliseconds of GPU time each frame gives at least the work drawn once in so many frames that may wait: the
  /// shadows' cascades and faces, the overhead maps, the surroundings' faces, the sky's refill. A frame held to a cap or
  /// the display's refresh gives it what headroom it leaves, up to `PeriodicBudget::CEILING`.
  pub periodic_budget: f32,
}

impl Default for RenderSettings {
  fn default() -> Self {
    Self {
      frame_rate: RenderFrameRate::default(),
      is_gpu_timed: false,
      graph: RenderGraphSettings::default(),
      backend: None,
      gpu: None,
      periodic_budget: RENDER_DEFAULT_PERIODIC_BUDGET,
    }
  }
}
