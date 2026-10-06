use serde::{Deserialize, Serialize};

use crate::contract::render_contact_shadow_mode::RenderContactShadowMode;

/// Local lights a pixel marches contact shadows towards at most.
pub const RENDER_MAX_CONTACT_SHADOW_LIGHTS: u32 = 8;

/// Contact shadows: the small shadows the sun's cascades and the lights' maps are too coarse to cast, or the lights
/// without a map do not cast at all, found in the frame's depth.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderContactShadowSettings {
  pub mode: RenderContactShadowMode,
  /// Metres each pixel's ray reaches towards the sun, or towards a light as far as the light at most.
  pub length: f32,
  /// How much of the light what the ray meets takes away: one all of it.
  pub intensity: f32,
  /// Metres behind what the depth shows that it is taken to be solid, near the camera; it grows with distance.
  pub thickness: f32,
  /// Depth reads along each ray.
  pub steps: u32,
  /// Local lights each pixel marches towards at most, the strongest there: none for the sun's alone.
  pub lights: u32,
}

impl Default for RenderContactShadowSettings {
  /// Off: the engine draws none.
  fn default() -> Self {
    Self {
      mode: RenderContactShadowMode::Engine,
      length: 0.6,
      intensity: 1.0,
      thickness: 0.1,
      steps: 16,
      lights: 4,
    }
  }
}

impl RenderContactShadowSettings {
  /// Whether they are drawn at all: enhanced, and each strength above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderContactShadowMode::Enhanced
      && self.length > 0.0
      && self.intensity > 0.0
      && self.thickness > 0.0
      && self.steps > 0
  }

  /// Local lights each pixel marches towards: none while they are not drawn, and never past the most there are.
  pub fn get_light_count(&self) -> u32 {
    if self.is_drawn() {
      self.lights.min(RENDER_MAX_CONTACT_SHADOW_LIGHTS)
    } else {
      0
    }
  }
}
