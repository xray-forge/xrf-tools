use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_method::RenderAmbientOcclusionMethod;
use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::contract::render_ambient_occlusion_vbao_settings::RenderAmbientOcclusionVbaoSettings;

/// Ambient occlusion from the depth of the frame at half resolution: it darkens the hemisphere and ambient light over
/// the baked hemisphere occlusion, as the engine's SSAO does. The radius, strength and quality are every method's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientOcclusionSettings {
  pub is_enabled: bool,
  /// GTAO's horizons, or VBAO's visibility bitmask, which `vbao` shapes.
  pub method: RenderAmbientOcclusionMethod,
  /// Metres around a point that what stands there occludes it from.
  pub radius: f32,
  /// How dark the occlusion goes: one the method's own curve, zero none, two its square.
  pub strength: f32,
  pub quality: RenderAmbientOcclusionQuality,
  pub vbao: RenderAmbientOcclusionVbaoSettings,
}

impl Default for RenderAmbientOcclusionSettings {
  /// XeGTAO's own, at a metre.
  fn default() -> Self {
    Self {
      is_enabled: true,
      method: RenderAmbientOcclusionMethod::Gtao,
      radius: 1.0,
      strength: 1.0,
      quality: RenderAmbientOcclusionQuality::High,
      vbao: RenderAmbientOcclusionVbaoSettings::default(),
    }
  }
}

impl RenderAmbientOcclusionSettings {
  /// Whether VBAO searches it rather than GTAO.
  pub fn is_vbao(&self) -> bool {
    self.method == RenderAmbientOcclusionMethod::Vbao
  }
}
