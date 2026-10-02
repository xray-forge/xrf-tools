use serde::{Deserialize, Serialize};

use crate::contract::render_light_shadow_filter::RenderLightShadowFilter;

/// The level's local lights: binned into clusters of the view, and accumulated after the sun in one pass.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLightsSettings {
  pub is_enabled: bool,
  /// Whether the level file's own lights are drawn too, which the engine does only with `r2_allow_r1_lights`.
  pub is_level_lights: bool,
  /// Whether a light the engine shadows casts its shadows.
  pub is_shadowed: bool,
  pub shadow_filter: RenderLightShadowFilter,
}

impl Default for RenderLightsSettings {
  /// The engine's own: every spawned light, and none of the level file's.
  fn default() -> Self {
    Self {
      is_enabled: true,
      is_level_lights: false,
      is_shadowed: true,
      shadow_filter: RenderLightShadowFilter::Engine,
    }
  }
}
