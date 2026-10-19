use serde::{Deserialize, Serialize};

use crate::contract::render_rain_drops_mode::RenderRainDropsMode;
use crate::contract::render_rain_mode::RenderRainMode;

/// How rain falls and wets the frame's surfaces: the engine's streaks or refracting drops, the engine's own wetting,
/// and whether puddles gather over it, with their strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRainSettings {
  pub mode: RenderRainMode,
  /// How the rain falls, apart from how it wets.
  pub drops: RenderRainDropsMode,
  /// How many puddles the level's sites hold: none at nought, about seven in ten at one, nearly all at two.
  pub puddles: f32,
  /// How much of a reflection a puddle takes at most.
  pub reflectivity: f32,
  /// How strongly rain ripples wet surfaces and puddles.
  pub ripples: f32,
  /// Metres out to which puddles are drawn, fading over the last sixth.
  pub distance: f32,
  /// Whether the puddles fill the relief where the enhanced parallax marches the ground: water in its hollows first,
  /// and the puddles' shores along them.
  pub is_relief_flooded: bool,
}

impl Default for RenderRainSettings {
  /// The engine's streaks and own wetting, with the enhanced wetting's designed strengths.
  fn default() -> Self {
    Self {
      mode: RenderRainMode::Engine,
      drops: RenderRainDropsMode::Engine,
      puddles: 0.5,
      reflectivity: 0.4,
      ripples: 1.0,
      distance: 300.0,
      is_relief_flooded: true,
    }
  }
}

impl RenderRainSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderRainMode::Enhanced
  }

  pub fn is_drops_enhanced(&self) -> bool {
    self.drops == RenderRainDropsMode::Enhanced
  }
}
