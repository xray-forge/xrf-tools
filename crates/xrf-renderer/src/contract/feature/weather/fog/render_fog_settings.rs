use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::weather::fog::render_fog_colour::RenderFogColour;
use crate::contract::feature::weather::fog::render_fog_mode::RenderFogMode;

/// How the fog is drawn beyond the weather's own keys, and the enhanced fog's strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFogSettings {
  // The enhanced fog's own menu bounds.
  pub mode: RenderFogMode,
  /// Metres either side of the world's zero height over which the height fog rises from none, above, to whole, below.
  #[setting(min = 1, max = 20)]
  pub height: f32,
  /// How much thicker the distance fog grows where the height fog is whole: one doubles it.
  #[setting(min = 0, max = 5)]
  pub density: f32,
  /// How far the height fog takes the sun's colour where the view faces the sun.
  #[setting(min = 0, max = 1)]
  pub sun_color: f32,
  /// How readily the bright parts of the frame are blurred into the fog: none blurs nothing.
  #[setting(min = 0, max = 1)]
  pub scattering: f32,
  /// The fog's share past which fogged surfaces fade into the sky behind them, whole where the fog is: none fades them
  /// by the fog's square from the first, as the engine does, which shows the sky through the middle distance.
  // The renderer's own: the fog's share past which far surfaces fade into the sky, none for the engine's square.
  #[setting(min = 0, max = 0.95)]
  pub sky_fade: f32,
  /// What colour the fog takes at the anchored light levels.
  pub colour: RenderFogColour,
}

impl Default for RenderFogSettings {
  /// The engine's own fog, with the enhanced fog's strengths but no scattering, which haloes what stands in fog, and at
  /// the anchored light levels half the weather's colour and half the sky's.
  fn default() -> Self {
    Self {
      mode: RenderFogMode::Engine,
      height: 8.0,
      density: 0.4,
      sun_color: 0.1,
      scattering: 0.0,
      sky_fade: 0.75,
      colour: RenderFogColour::default(),
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
