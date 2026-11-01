use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::weather::render_volumetric_mode::RenderVolumetricMode;

/// How the light the air scatters is drawn, and the enhanced volume's strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderVolumetricSettings {
  // The renderer's own bounds: up to four times the designed scattering, a phase function short of a beam, and
  // from a nearby grid to 300 m.
  pub mode: RenderVolumetricMode,
  /// How strongly the enhanced volume scatters, one as designed.
  #[setting(min = 0, max = 4)]
  pub intensity: f32,
  /// How far the scattered light leans towards where it was going, from none (every way alike) to nearly all.
  #[setting(min = 0, max = 0.9)]
  pub anisotropy: f32,
  /// Metres along the view the enhanced volume reaches.
  #[setting(min = 25, max = 300)]
  pub distance: f32,
}

impl Default for RenderVolumetricSettings {
  /// The engine's shafts, with the enhanced volume at its designed strength and a quarter's lean for when it is drawn.
  fn default() -> Self {
    Self {
      mode: RenderVolumetricMode::Engine,
      intensity: 1.0,
      anisotropy: 0.25,
      distance: 120.0,
    }
  }
}

impl RenderVolumetricSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderVolumetricMode::Enhanced && self.intensity > 0.0 && self.distance > 0.0
  }
}
