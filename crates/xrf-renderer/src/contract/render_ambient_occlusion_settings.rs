use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;

/// Ambient occlusion from the depth of the frame, GTAO as XeGTAO computes it at half resolution: it darkens the
/// hemisphere and ambient light over the baked hemisphere occlusion, as the engine's SSAO does.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientOcclusionSettings {
  pub is_enabled: bool,
  /// Metres around a point that what stands there occludes it from.
  pub radius: f32,
  /// How dark the occlusion goes: one XeGTAO's own curve, zero none, two its square.
  pub strength: f32,
  pub quality: RenderAmbientOcclusionQuality,
}

impl Default for RenderAmbientOcclusionSettings {
  /// XeGTAO's own, at a metre.
  fn default() -> Self {
    Self {
      is_enabled: true,
      radius: 1.0,
      strength: 1.0,
      quality: RenderAmbientOcclusionQuality::High,
    }
  }
}
