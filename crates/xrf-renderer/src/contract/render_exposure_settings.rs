use serde::{Deserialize, Serialize};

/// The engine's exposure (`r2_tonemap`): the frame's average luminance measured every frame, and the scale the tonemap
/// multiplies by moved towards `middle_gray / luminance` at the adaptation's rate.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderExposureSettings {
  /// Off, the tonemap multiplies by one, the engine's answer at noon.
  pub is_enabled: bool,
  /// `r2_tonemap_amount`: how far from no adaptation towards the whole of it.
  pub amount: f32,
  /// `r2_tonemap_middlegray`: the luminance the frame is brought towards.
  pub middle_gray: f32,
  /// `r2_tonemap_lowlum`: what the luminance is floored at, so a black frame is not brightened without end.
  pub low_luminance: f32,
  /// `r2_tonemap_adaptation`: how fast the scale follows the frame.
  pub adaptation: f32,
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
    }
  }
}
