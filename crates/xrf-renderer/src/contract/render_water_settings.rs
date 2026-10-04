use serde::{Deserialize, Serialize};

/// The water (`water.vs`, `water.ps`, `waterd.ps`): rippled and reflecting the sky, blended over the depth behind it
/// and distorting it. The engine's own look by default: its constants are `shared/waterconfig.h`'s and `def_distort`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWaterSettings {
  /// Off, what lies under the water shows.
  pub is_enabled: bool,
  /// `r2_soft_water`: soft water fades by the depth behind it, darkens with it and lays foam in the shallows.
  pub is_soft: bool,
  /// Whether water writes the distortion it causes, moving what is seen through it.
  pub is_distorted: bool,
  /// How high the waves lift the surface, in metres: `W_POSITION_SHIFT_HEIGHT`.
  pub wave_height: f32,
  /// How fast they run: `W_POSITION_SHIFT_SPEED`.
  pub wave_speed: f32,
  /// What the two normal layers' scroll is multiplied by, one as the engine scrolls them.
  pub ripple: f32,
  /// What the sky's reflection is multiplied by, one as the engine mixes it.
  pub reflection: f32,
  /// How far the distortion target moves what is behind it, a share of the screen: `def_distort`, which moves what
  /// the distorting particles write as well.
  pub distortion: f32,
}

impl Default for RenderWaterSettings {
  /// The engine's own water.
  fn default() -> Self {
    Self {
      is_enabled: true,
      is_soft: true,
      is_distorted: true,
      wave_height: 1.0 / 60.0,
      wave_speed: 25.0,
      ripple: 1.0,
      reflection: 1.0,
      distortion: 0.05,
    }
  }
}
