/// How wet the level has become: built up towards the rain's density while it rains, as Monolith's weather accumulates
/// `wetness_factor`, and dried slowly towards it after, on the weather's clock; dry before any rain, as every rain starts
/// on dry ground, so surfaces wet and dry smoothly and settle at the density the engine wets by.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RainWetness {
  /// Its wetness, from nothing to one, and the clock it was last moved on at.
  state: Option<(f32, f32)>,
}

impl RainWetness {
  /// What a real second of rain at full density adds in the game: Monolith's `density / 10000` a frame, at sixty frames.
  pub const ENGINE_BUILD_UP: f32 = 0.006;
  /// The enhanced rain's: twice the engine's, so surfaces wet in step with a rain's first minutes, the puddles still
  /// filling at their own pace.
  pub const ENHANCED_BUILD_UP: f32 = Self::ENGINE_BUILD_UP * 2.0;
  /// What a dry real second takes away in the game: Monolith's `0.0001 × 0.3` a frame, at sixty frames.
  const DRYING: f32 = 0.0018;

  /// Moves it on to the real clock `time`, in seconds, under a rain of `density`, its weather running `pace` times as
  /// fast as the game's own clock does (the weather's clock rate over the game's `time_factor`), building up by
  /// `build_up` a second of full rain, and returns it.
  pub fn advance(&mut self, time: f32, density: f32, pace: f32, build_up: f32) -> f32 {
    let density: f32 = density.clamp(0.0, 1.0);
    let wetness: f32 = match self.state {
      None => 0.0,
      Some((wetness, clock)) => {
        let elapsed: f32 = (time - clock).max(0.0) * pace.max(0.0);

        if wetness < density {
          (wetness + density * build_up * elapsed).min(density)
        } else {
          (wetness - Self::DRYING * elapsed).max(density)
        }
      }
    };

    self.state = Some((wetness, time));

    wetness
  }

  /// Its wetness, nothing before it was first moved on.
  pub fn get(&self) -> f32 {
    self.state.map_or(0.0, |(wetness, _)| wetness)
  }
}
