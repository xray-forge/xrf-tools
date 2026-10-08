use serde::{Deserialize, Serialize};

use crate::contract::render_rain_mode::RenderRainMode;

/// How rain wets the frame's surfaces, and the enhanced wetting's strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRainSettings {
  pub mode: RenderRainMode,
  /// How much of flat terrain puddles may cover, from none to most of it.
  pub puddles: f32,
  /// How much of a reflection a puddle takes at most.
  pub reflectivity: f32,
  /// How strongly rain ripples wet surfaces and puddles.
  pub ripples: f32,
}

impl Default for RenderRainSettings {
  /// The engine's own wetting, with the enhanced wetting's designed strengths.
  fn default() -> Self {
    Self {
      mode: RenderRainMode::Engine,
      puddles: 0.8,
      reflectivity: 0.4,
      ripples: 1.0,
    }
  }
}

impl RenderRainSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderRainMode::Enhanced
  }
}
