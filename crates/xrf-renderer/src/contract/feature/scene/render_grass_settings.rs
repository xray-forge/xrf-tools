use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::scene::render_foliage_settings::RenderFoliageSettings;

/// Metres grass is planted to at most, whatever a setting asks: past it the planting's buffers outgrow any device.
pub const RENDER_MAX_GRASS_RADIUS: f32 = 512.0;

/// The grass (`CDetailManager`): planted on the GPU around the camera as the engine plants it, and drawn into the
/// G-buffer. The engine's are 49 metres round at a density of 0.6 (`r__detail_radius`, `r__detail_density`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGrassSettings {
  pub is_enabled: bool,
  /// How far apart a slot's candidates stand, from 0.1 (the densest) to 0.99 (the sparsest): `r__detail_density`.
  // `r__detail_density`: a spacing. The console goes to 0.1, six times the game's; past three times it at the
  // widest radius the grass's buffers outgrow what the GPU process holds and the renderer crashes.
  #[setting(min = 0.2, max = 0.99)]
  pub density: f32,
  /// Whole metres around the camera grass is planted to: `r__detail_radius`.
  #[setting(min = 49, max = 450, integer, wasted_above = 300)]
  pub radius: f32,
  /// What every planted tuft is scaled by: `r__detail_height`.
  #[setting(min = 0.5, max = 2)]
  pub height: f32,
  /// How the trees and the grass move in the wind.
  pub foliage: RenderFoliageSettings,
  /// Degrees of slope past which a tuft is not planted, thinning over the last fifth: ninety for none, as the engine
  /// plants on any ground.
  // The renderer's own: degrees of slope, ninety for none.
  #[setting(min = 10, max = 90)]
  pub slope_limit: f32,
  /// Whether a tuft stands along its ground's normal rather than straight up.
  pub is_aligned: bool,
  /// Whether the grass casts shadows at all: the contact shadows' within `shadow_radius`, and as cards (`GrassCards`)
  /// the sun's and the lamps' traced under ray reconstruction or a traced occlusion, the sun's drawn into its
  /// `shadow_cascades` nearest cascades otherwise, as Screen Space Shaders' `ssfx_grass_shadows` draws them. Neither
  /// engine shadows its grass.
  pub is_shadowed: bool,
  /// Metres from the camera the grass casts shadows to, 150 at most, fading over the last fifth of them: the cards' as
  /// far as their cascades reach, the contact shadows' past them.
  #[setting(min = 5, max = 150)]
  pub shadow_radius: f32,
  /// How many of the sun's nearest cascades the cards are drawn into: none for the contact shadows alone.
  // The renderer's own: the fourth cascade starts past the 150 m the grass casts to at most, so drawing into it
  // buys nothing.
  #[setting(min = 0, max = 4, recommended = [0, 3], wasted_above = 3)]
  pub shadow_cascades: u32,
}

impl Default for RenderGrassSettings {
  /// The engine's own grass.
  fn default() -> Self {
    Self {
      is_enabled: true,
      density: 0.6,
      radius: 49.0,
      height: 1.0,
      foliage: RenderFoliageSettings::default(),
      slope_limit: 90.0,
      is_aligned: false,
      is_shadowed: false,
      shadow_radius: 50.0,
      shadow_cascades: 0,
    }
  }
}

impl RenderGrassSettings {
  /// Metres grass is planted to, from nought to `RENDER_MAX_GRASS_RADIUS`; nought for one not a number.
  pub fn get_radius(&self) -> f32 {
    if self.radius.is_nan() {
      0.0
    } else {
      self.radius.clamp(0.0, RENDER_MAX_GRASS_RADIUS)
    }
  }
}
