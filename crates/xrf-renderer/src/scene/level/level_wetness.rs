use crate::contract::render_rain_settings::RenderRainSettings;
use crate::lighting::render_lighting::RenderLighting;
use crate::scene::level::puddle_fill::PuddleFill;
use crate::scene::level::rain_wetness::RainWetness;

/// The level's wetness and its puddles' fill: the scene's own, moved on once a frame by its driving viewport's rain and
/// clock, which every view of it wets its surfaces by.
#[derive(Default)]
pub struct LevelWetness {
  wetness: RainWetness,
  puddle_fill: PuddleFill,
}

impl LevelWetness {
  /// The game's own pace, which the level wets and dries at.
  const PACE: f32 = 1.0;

  /// Moves both on to the scene's real clock `time` under the rain `lighting` holds, at the game's own pace whatever
  /// the weather's clock runs at, building up as the rain's `settings` say.
  pub fn advance(&mut self, lighting: &RenderLighting, settings: &RenderRainSettings, time: f32) {
    let density: f32 = lighting.rain.map_or(0.0, |rain| rain.density);
    let build_up: f32 = if settings.is_enhanced() {
      RainWetness::ENHANCED_BUILD_UP
    } else {
      RainWetness::ENGINE_BUILD_UP
    };

    self.wetness.advance(time, density, Self::PACE, build_up);
    self.puddle_fill.advance(time, density, Self::PACE);
  }

  /// How wet the level is, from nothing to one.
  pub fn get_wetness(&self) -> &RainWetness {
    &self.wetness
  }

  /// How full its puddles are, how stormy the rain has been and how soaked the level is.
  pub fn get_puddles(&self) -> (f32, f32, f32) {
    self.puddle_fill.get()
  }
}
