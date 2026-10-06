/// How wet the level has become: built up while it rains, by how hard, and dried slowly after, as Monolith's weather
/// accumulates `wetness_factor`, on the weather's clock; already as wet as the rain's density where a view first sees
/// it rain.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RainWetness {
  /// Its wetness, from nothing to one, and the clock it was last moved on at.
  state: Option<(f32, f32)>,
}

impl RainWetness {
  /// What a real second of rain at full density adds in the game: Monolith's `density / 10000` a frame, at sixty frames.
  const BUILD_UP: f32 = 0.006;
  /// What a dry real second takes away in the game: Monolith's `0.0001 × 0.3` a frame, at sixty frames.
  const DRYING: f32 = 0.0018;

  /// Moves it on to the real clock `time`, in seconds, under a rain of `density`, its weather running `pace` times as
  /// fast as the game's own clock does (the weather's clock rate over the game's `time_factor`), and returns it.
  pub fn advance(&mut self, time: f32, density: f32, pace: f32) -> f32 {
    let density: f32 = density.clamp(0.0, 1.0);
    let wetness: f32 = match self.state {
      None => density,
      Some((wetness, clock)) => {
        let elapsed: f32 = (time - clock).max(0.0) * pace.max(0.0);
        let change: f32 = if density > 0.0 {
          density * Self::BUILD_UP
        } else {
          -Self::DRYING
        };

        (wetness + change * elapsed).clamp(0.0, 1.0)
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
