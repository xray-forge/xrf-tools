use serde::{Deserialize, Serialize};

use crate::contract::render_fog_mode::RenderFogMode;

/// How the fog is drawn beyond the weather's own keys, and the enhanced fog's strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFogSettings {
  pub mode: RenderFogMode,
  /// Metres either side of the world's zero height over which the height fog rises from none, above, to whole, below.
  pub height: f32,
  /// How much thicker the distance fog grows where the height fog is whole: one doubles it.
  pub density: f32,
  /// How far the height fog takes the sun's colour where the view faces the sun.
  pub sun_color: f32,
  /// How readily the bright parts of the frame are blurred into the fog: none blurs nothing.
  pub scattering: f32,
}

impl Default for RenderFogSettings {
  /// The engine's own fog, with the enhanced fog's shipped strengths.
  fn default() -> Self {
    Self {
      mode: RenderFogMode::Engine,
      height: 8.0,
      density: 1.3,
      sun_color: 0.1,
      scattering: 0.7,
    }
  }
}

impl RenderFogSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderFogMode::Enhanced
  }

  /// Whether the frame's light is scattered into the fog: enhanced, and its scattering above none.
  pub fn is_scattered(&self) -> bool {
    self.is_enhanced() && self.scattering > 0.0
  }
}
