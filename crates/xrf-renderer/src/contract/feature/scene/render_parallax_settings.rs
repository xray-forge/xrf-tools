use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::scene::render_parallax_mode::RenderParallaxMode;

/// How the level's surfaces are displaced by their height.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParallaxSettings {
  // The renderer's own bounds: the third-party pack's menu bounds, its height doubled.
  pub mode: RenderParallaxMode,
  /// `r2_steep_parallax`: the engines' steep parallax on a level surface whose texture asks for it.
  pub is_steep: bool,
  /// Steps an enhanced march takes at most, looking along a surface; three looking straight down at it.
  #[setting(min = 1, max = 64, recommended = [8, 32])]
  pub steps: u32,
  /// Metres within which the enhanced parallax is marched, faded over the last fifth of them.
  #[setting(min = 0, max = 100)]
  pub range: f32,
  /// How deep the enhanced parallax reaches, in the detail's texture coordinates.
  #[setting(min = 0, max = 0.2)]
  pub height: f32,
  /// Whether an enhanced march settles where it crossed the surface by halving its last step three times.
  pub is_refined: bool,
  /// How much of the sun the enhanced parallax's own relief keeps from its hollows: none to all.
  #[setting(min = 0, max = 1)]
  pub shadow: f32,
  /// How far a terrain's layers blend by height, the one standing highest showing through: none, the mask alone, to
  /// all.
  #[setting(min = 0, max = 1)]
  pub blending: f32,
  /// How much the enhanced parallax darkens the low ground of its relief: none to its whole share.
  #[setting(min = 0, max = 2)]
  pub cavity: f32,
  /// How deep the enhanced parallax reaches on a level surface whose texture asks for parallax, in its own texture
  /// coordinates, and the metres within which it is marched there.
  #[setting(min = 0, max = 0.1)]
  pub surface_height: f32,
  #[setting(min = 0, max = 100)]
  pub surface_range: f32,
}

impl Default for RenderParallaxSettings {
  /// The engines' own, steep as Anomaly's high and extreme presets set `r2_steep_parallax`, with the enhanced
  /// parallax's strengths for when it is drawn: eight steps within 60 m, 0.02 of the detail deep, refined,
  /// self-shadowed, its layers half blended by height.
  fn default() -> Self {
    Self {
      mode: RenderParallaxMode::Engine,
      is_steep: true,
      steps: 8,
      range: 60.0,
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
