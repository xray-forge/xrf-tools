use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

/// The engine's exposure (`r2_tonemap`): the frame's average luminance measured every frame, and the scale the tonemap
/// multiplies by moved towards `middle_gray / luminance` at the adaptation's rate.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderExposureSettings {
  // The console's own bounds.
  /// Off, the tonemap multiplies by one, the engine's answer at noon.
  pub is_enabled: bool,
  /// `r2_tonemap_amount`: how far from no adaptation towards the whole of it.
  #[setting(min = 0, max = 1)]
  pub amount: f32,
  /// `r2_tonemap_middlegray`: the luminance the frame is brought towards.
  #[setting(min = 0, max = 2)]
  pub middle_gray: f32,
  /// `r2_tonemap_lowlum`: what the luminance is floored at, so a black frame is not brightened without end.
  #[setting(min = 0.0001, max = 1)]
  pub low_luminance: f32,
  /// `r2_tonemap_adaptation`: how fast the scale follows the frame.
  #[setting(min = 0.01, max = 10)]
  pub adaptation: f32,
  /// Stops the physical light's exposure lifts the frame over the level it meters it at, or lowers it where negative;
  /// the engine's exposure has no such setting.
  // Stops either way, as a camera's exposure compensation dial.
  #[setting(min = -6, max = 6)]
  pub compensation: f32,
}

impl Default for RenderExposureSettings {
  /// OpenXRay's own.
  fn default() -> Self {
    Self {
      is_enabled: true,
      amount: 0.7,
      middle_gray: 1.0,
      low_luminance: 0.0001,
      adaptation: 1.0,
      compensation: 0.0,
    }
  }
}
