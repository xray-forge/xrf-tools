use serde::{Deserialize, Serialize};
use xrf_renderer_core::RenderGpu;
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::settings::gpu::render_backend::RenderBackend;
use crate::contract::settings::render_frame_rate::RenderFrameRate;
use crate::contract::settings::render_graph_settings::RenderGraphSettings;
use crate::contract::view::render_texture_filtering::RenderTextureFiltering;

/// GPU milliseconds a frame gives the work drawn once in so many frames that may wait, unless the settings say
/// otherwise.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const RENDER_DEFAULT_PERIODIC_BUDGET: f32 = 0.5;

/// What every viewport of the renderer draws with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
  // The textures' filtering and quality alone are settings a preset chooses; the rest is the device's.
  #[setting(skip)]
  pub frame_rate: RenderFrameRate,
  /// Whether each pass of a frame is timed on the GPU, where the device writes timestamps between passes.
  #[setting(skip)]
  pub is_gpu_timed: bool,
  /// Which of the frame graph's mechanisms the frames compile with.
  #[setting(skip)]
  pub graph: RenderGraphSettings,
  /// The graphics API to draw with, none for the first that starts; one that cannot start falls back to the first that
  /// does, and `XRF_RENDER_BACKEND` overrides it.
  #[setting(skip)]
  pub backend: Option<RenderBackend>,
  /// The GPU to draw on, none for the one the backend picks automatically (its high-performance adapter); one the
  /// backend does not have falls back to automatic.
  #[setting(skip)]
  pub gpu: Option<RenderGpu>,
  /// Milliseconds of GPU time each frame gives at least the work drawn once in so many frames that may wait: the
  /// shadows' cascades and faces, the overhead maps, the surroundings' faces, the sky's refill. A frame held to a cap
  /// or the display's refresh gives it what headroom it leaves, up to `PeriodicBudget::CEILING`.
  #[setting(skip)]
  pub periodic_budget: f32,
  /// How the level's surfaces sample their textures, which every viewport shares.
  pub texture_filtering: RenderTextureFiltering,
  /// `texture_lod`, nought to four: how much coarser than stored the textures load, the engine's texture quality,
  /// which every viewport shares.
  #[setting(min = 0, max = 4)]
  pub texture_lod: u32,
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
      texture_filtering: RenderTextureFiltering::default(),
      texture_lod: 0,
    }
  }
}
