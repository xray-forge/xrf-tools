use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::weather::water::render_enhanced_water_settings::RenderEnhancedWaterSettings;
use crate::contract::feature::weather::water::render_water_mode::RenderWaterMode;

/// The water (`water.vs`, `water.ps`, `waterd.ps`): rippled and reflecting the sky, blended over the depth behind it
/// and distorting it. The engine's own look by default: its constants are `shared/waterconfig.h`'s and `def_distort`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWaterSettings {
  /// Off, what lies under the water shows.
  pub is_enabled: bool,
  /// The engine's water, or the enhanced water, which `enhanced` shapes.
  pub mode: RenderWaterMode,
  /// `r2_soft_water`: soft water fades by the depth behind it, darkens with it and lays foam in the shallows.
  pub is_soft: bool,
  /// Whether water writes the distortion it causes, moving what is seen through it.
  pub is_distorted: bool,
  /// How high the waves lift the engine's water, in metres: `W_POSITION_SHIFT_HEIGHT`, 1/60 on the setting's step. The
  /// enhanced water stands flat, its waves in its maps.
  #[setting(min = 0, max = 0.2)]
  pub wave_height: f32,
  /// How fast they run: `W_POSITION_SHIFT_SPEED`.
  #[setting(min = 0, max = 100)]
  pub wave_speed: f32,
  /// What the two normal layers' scroll is multiplied by, one as the engine scrolls them.
  #[setting(min = 0, max = 4)]
  pub ripple: f32,
  /// What the sky's reflection is multiplied by, one as the engine mixes it.
  #[setting(min = 0, max = 4)]
  pub reflection: f32,
  /// How far the distortion target moves what is behind it, a share of the screen: `def_distort`, which moves what
  /// the distorting particles write as well.
  #[setting(min = 0, max = 0.2)]
  pub distortion: f32,
  pub enhanced: RenderEnhancedWaterSettings,
}

impl Default for RenderWaterSettings {
  /// The engine's own water: `shared/waterconfig.h`'s constants and `def_distort`.
  fn default() -> Self {
    Self {
      is_enabled: true,
      mode: RenderWaterMode::Engine,
      is_soft: true,
      is_distorted: true,
      wave_height: 0.017,
      wave_speed: 25.0,
      ripple: 1.0,
      reflection: 1.0,
      distortion: 0.05,
      enhanced: RenderEnhancedWaterSettings::default(),
    }
  }
}
