use serde::{Deserialize, Serialize};

use crate::contract::render_rain_mode::RenderRainMode;

/// How rain wets the frame's surfaces: the engine's own wetting, and whether puddles gather over it, with their
/// strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRainSettings {
  pub mode: RenderRainMode,
  /// How many puddles the level's sites hold: none at nought, about seven in ten at one, nearly all at two.
  pub puddles: f32,
  /// How much of a reflection a puddle takes at most.
  pub reflectivity: f32,
  /// How strongly rain ripples wet surfaces and puddles.
  pub ripples: f32,
  /// Metres out to which puddles are drawn, fading over the last sixth.
  pub distance: f32,
}

impl Default for RenderRainSettings {
  /// The engine's own wetting, with the enhanced wetting's designed strengths.
  fn default() -> Self {
    Self {
      mode: RenderRainMode::Engine,
      puddles: 1.0,
      reflectivity: 0.4,
      ripples: 1.0,
      distance: 300.0,
    }
  }
}

impl RenderRainSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderRainMode::Enhanced
  }
}
