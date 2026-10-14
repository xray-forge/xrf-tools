use serde::{Deserialize, Serialize};

use crate::contract::render_foliage_settings::RenderFoliageSettings;

/// The grass (`CDetailManager`): planted on the GPU around the camera as the engine plants it, and drawn into the
/// G-buffer. The engine's are 49 metres round at a density of 0.6 (`r__detail_radius`, `r__detail_density`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGrassSettings {
  pub is_enabled: bool,
  /// How far apart a slot's candidates stand, from 0.1 (the densest) to 0.99 (the sparsest): `r__detail_density`.
  pub density: f32,
  /// Whole metres around the camera grass is planted to: `r__detail_radius`.
  pub radius: f32,
  /// What every planted tuft is scaled by: `r__detail_height`.
  pub height: f32,
  /// How the trees and the grass move in the wind.
  pub foliage: RenderFoliageSettings,
  /// Degrees of slope past which a tuft is not planted, thinning over the last fifth: ninety for none, as the engine
  /// plants on any ground.
  pub slope_limit: f32,
  /// Whether a tuft stands along its ground's normal rather than straight up.
  pub is_aligned: bool,
  /// Whether the grass casts shadows near the camera, as cards (`GrassCards`): the sun's and the lamps' traced under
  /// ray reconstruction or a traced occlusion, the sun's drawn into its two nearest cascades otherwise, as Screen Space
  /// Shaders' `ssfx_grass_shadows` draws them. Neither engine shadows its grass.
  pub is_shadowed: bool,
  /// Metres from the camera the grass casts shadows to, fading over the last fifth of them.
  pub shadow_radius: f32,
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
      shadow_radius: 25.0,
    }
  }
}
