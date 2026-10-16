use serde::{Deserialize, Serialize};

use crate::contract::render_volumetric_mode::RenderVolumetricMode;

/// How the light the air scatters is drawn, and the enhanced volume's strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderVolumetricSettings {
  pub mode: RenderVolumetricMode,
  /// How strongly the enhanced volume scatters, one as designed.
  pub intensity: f32,
  /// How far the scattered light leans towards where it was going, from none (every way alike) to nearly all.
  pub anisotropy: f32,
  /// Metres along the view the enhanced volume reaches.
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
