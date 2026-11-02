use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

/// RTAO's trace: the rays each point casts a frame.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientOcclusionRtaoSettings {
  #[setting(min = 1, max = 8, recommended = [2, 8], wasted_above = 4)]
  pub rays: u32,
}

impl Default for RenderAmbientOcclusionRtaoSettings {
  /// Four rays a point, accumulated over VBAO's frames.
  fn default() -> Self {
    Self { rays: 4 }
  }
}

impl RenderAmbientOcclusionRtaoSettings {
  /// The least and most rays a point casts.
  pub const RAYS_RANGE: (u32, u32) = (1, 8);

  pub fn get_rays(&self) -> u32 {
    self.rays.clamp(Self::RAYS_RANGE.0, Self::RAYS_RANGE.1)
  }
}
