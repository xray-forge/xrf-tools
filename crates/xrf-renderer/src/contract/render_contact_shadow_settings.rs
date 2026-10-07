use serde::{Deserialize, Serialize};

use crate::contract::render_contact_shadow_mode::RenderContactShadowMode;

/// Contact shadows: the small shadows the sun's cascades are too coarse to cast, found in the frame's depth.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderContactShadowSettings {
  pub mode: RenderContactShadowMode,
  /// Metres each pixel's ray reaches towards the sun.
  pub length: f32,
  /// How much of the sunlight what the ray meets takes away: one all of it.
  pub intensity: f32,
  /// Metres behind what the depth shows that it is taken to be solid, near the camera; it grows with distance.
  pub thickness: f32,
  /// Depth reads along each ray.
  pub steps: u32,
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
}
