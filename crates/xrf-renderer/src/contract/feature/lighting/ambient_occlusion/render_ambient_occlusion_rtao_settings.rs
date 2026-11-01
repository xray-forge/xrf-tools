use serde::{Deserialize, Serialize};

/// RTAO's trace: the rays each point casts a frame.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientOcclusionRtaoSettings {
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
