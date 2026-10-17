use xrf_engine_target::XrayEngine;

use crate::contract::render_rain_settings::RenderRainSettings;
use crate::lighting::render_lighting::RenderLighting;
use crate::scene::level::puddle_fill::PuddleFill;
use crate::scene::level::rain_wetness::RainWetness;

/// The level's wetness and its puddles' fill: the scene's own, moved on once a frame by its first view's rain and
/// clock, which every view of it wets its surfaces by.
#[derive(Default)]
pub struct LevelWetness {
  wetness: RainWetness,
  puddle_fill: PuddleFill,
}

impl LevelWetness {
  /// Moves both on to the scene's clock `time` under the rain `lighting` holds, its weather's clock running `rate`
  /// times real time, building up as the rain's `settings` say.
  pub fn advance(&mut self, lighting: &RenderLighting, settings: &RenderRainSettings, (time, rate): (f32, f32)) {
    // The game's `time_factor`: Anomaly's `alife.ltx` runs its clock six times a real second, Call of Pripyat's ten.
    let time_factor: f32 = if lighting.engine == XrayEngine::Extended {
      6.0
    } else {
      10.0
    };
    let density: f32 = lighting.rain.map_or(0.0, |rain| rain.density);
    // A paused clock still lets the level wet and dry at the game's own pace, so a rain switched on or off in a view
    // standing at a moment fills and drains as it would in play.
    let pace: f32 = if rate > 0.0 { rate / time_factor } else { 1.0 };
    let build_up: f32 = if settings.is_enhanced() {
      RainWetness::ENHANCED_BUILD_UP
    } else {
      RainWetness::ENGINE_BUILD_UP
    };

    self.wetness.advance(time, density, pace, build_up);
    self.puddle_fill.advance(time, density, pace);
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
