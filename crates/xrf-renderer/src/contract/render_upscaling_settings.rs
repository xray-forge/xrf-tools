use serde::{Deserialize, Serialize};

use crate::contract::render_dlss_preset::RenderDlssPreset;
use crate::contract::render_scale::RenderScale;

/// What the scene is drawn at, a share of the viewport upscaled to it, how sharply the upscaled frame is finished, and
/// which model DLSS upscales with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderUpscalingSettings {
  pub scale: RenderScale,
  /// RCAS's sharpness while upscaled, from none to its most.
  pub sharpening: f32,
  pub dlss_preset: RenderDlssPreset,
}

impl Default for RenderUpscalingSettings {
  fn default() -> Self {
    Self {
      scale: RenderScale::Native,
      sharpening: 0.5,
      dlss_preset: RenderDlssPreset::Default,
    }
  }
}

impl RenderUpscalingSettings {
  /// RCAS's `con`: FSR 2's remap of a sharpness to stops, `(1 - sharpening) * 2`, as `exp2(-stops)`.
  pub fn get_sharpness(&self) -> f32 {
    (-(1.0 - self.sharpening.clamp(0.0, 1.0)) * 2.0).exp2()
  }
}
