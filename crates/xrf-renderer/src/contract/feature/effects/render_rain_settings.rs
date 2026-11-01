use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::effects::render_rain_drops_mode::RenderRainDropsMode;
use crate::contract::feature::effects::render_rain_mode::RenderRainMode;

/// How rain falls and wets the frame's surfaces: the engine's streaks or refracting drops, the engine's own wetting,
/// and whether puddles gather over it, with their strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRainSettings {
  // The renderer's own bounds: puddles from none to nearly every site at two, a puddle's reflection at most a
  // mirror's, the rain's ripples up to twice their designed strength, and puddles drawn out to the widest grass's
  // radius, which the level surface they are placed on reaches past; the refracting drops' rings out to where a ring
  // is a pixel or two, up to three times their designed strength, the drops over a square from five metres to 150,
  // thinning out past 15, and from half their designed size, where their waves still span a few of the rings' texels,
  // to twice it; the level wetting and its puddles filling from the game's own pace to a hundred times it, where a
  // moderate rain fills them in seconds.
  pub mode: RenderRainMode,
  /// How the rain falls, apart from how it wets.
  pub drops: RenderRainDropsMode,
  /// Metres across the square about the camera the refracting drops fall over, thinning out past 15 metres.
  #[setting(min = 5, max = 150)]
  pub drop_square: f32,
  /// Metres across the ground out to which the refracting drops' rings ripple water and puddles, fading over the
  /// last fifth.
  #[setting(min = 0, max = 300)]
  pub ring_distance: f32,
  /// How strongly the refracting drops' rings ripple water and puddles, of their designed strength.
  #[setting(min = 0, max = 3)]
  pub ring_strength: f32,
  /// How wide the refracting drops' rings spread and how long their waves are, of their designed size.
  #[setting(min = 0.5, max = 2)]
  pub ring_size: f32,
  /// How many puddles the level's sites hold: none at nought, about seven in ten at one, nearly all at two.
  #[setting(min = 0, max = 2)]
  pub puddles: f32,
  /// How much of a reflection a puddle takes at most.
  #[setting(min = 0, max = 1)]
  pub reflectivity: f32,
  /// How strongly rain ripples wet surfaces and puddles.
  #[setting(min = 0, max = 2)]
  pub ripples: f32,
  /// How many times the game's own pace the level wets, dries and fills and drains its puddles at: one as the game
  /// does, more to see a rain's course in moments.
  #[setting(min = 1, max = 100, integer)]
  pub pace: f32,
  /// Metres out to which puddles are drawn, fading over the last sixth.
  #[setting(min = 30, max = 450, integer)]
  pub distance: f32,
  /// Whether the puddles fill the relief where the enhanced parallax marches the ground: water in its hollows first,
  /// and the puddles' shores along them.
  pub is_relief_flooded: bool,
}

impl Default for RenderRainSettings {
  /// The engine's streaks and own wetting; enhanced, half the puddle sites filled to 300 m, each taking up to 40%
  /// reflection; the refracting drops over 30 m about the camera, their rings out to 75 m.
  fn default() -> Self {
    Self {
      mode: RenderRainMode::Engine,
      drops: RenderRainDropsMode::Engine,
      drop_square: 30.0,
      ring_distance: 75.0,
      ring_strength: 1.0,
      ring_size: 1.0,
      puddles: 0.5,
      reflectivity: 0.4,
      ripples: 1.0,
      pace: 1.0,
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
