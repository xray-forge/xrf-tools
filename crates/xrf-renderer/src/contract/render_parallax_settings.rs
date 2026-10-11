use serde::{Deserialize, Serialize};

use crate::contract::render_parallax_mode::RenderParallaxMode;

/// How the level's surfaces are displaced by their height.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParallaxSettings {
  pub mode: RenderParallaxMode,
  /// `r2_steep_parallax`: the engines' steep parallax on a level surface whose texture asks for it.
  pub is_steep: bool,
  /// Steps an enhanced march takes at most, looking along a surface; three looking straight down at it.
  pub steps: u32,
  /// Metres within which the enhanced parallax is marched, faded over the last fifth of them.
  pub range: f32,
  /// How deep the enhanced parallax reaches, in the detail's texture coordinates.
  pub height: f32,
  /// Whether an enhanced march settles where it crossed the surface by halving its last step three times.
  pub is_refined: bool,
  /// How much of the sun the enhanced parallax's own relief keeps from its hollows: none to all.
  pub shadow: f32,
  /// How far a terrain's layers blend by height, the one standing highest showing through: none, the mask alone, to all.
  pub blending: f32,
  /// How much the enhanced parallax darkens the low ground of its relief: none to its whole share.
  pub cavity: f32,
  /// How deep the enhanced parallax reaches on a level surface whose texture asks for parallax, in its own texture
  /// coordinates, and the metres within which it is marched there.
  pub surface_height: f32,
  pub surface_range: f32,
}

impl Default for RenderParallaxSettings {
  /// The engines' own, steep as Anomaly's high and extreme presets set it, with the enhanced parallax's strengths for
  /// when it is drawn: on surfaces within the engines' 12 m.
  fn default() -> Self {
    Self {
      mode: RenderParallaxMode::Engine,
      is_steep: true,
      steps: 16,
      range: 20.0,
      height: 0.02,
      is_refined: true,
      shadow: 1.0,
      blending: 0.5,
      cavity: 1.0,
      surface_height: 0.02,
      surface_range: 60.0,
    }
  }
}

impl RenderParallaxSettings {
  /// Whether the enhanced parallax is marched.
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderParallaxMode::Enhanced && self.range > 0.0 && self.height > 0.0 && self.steps > 0
  }
}
