/// How full the level's puddles are, how stormy the rain has been and how soaked a long rain has left the level, on the
/// weather's clock, from empty and dry before any rain.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PuddleFill {
  /// How full, how stormy, how soaked, and the clock they were last moved on at.
  state: Option<(f32, f32, f32, f32)>,
}

impl PuddleFill {
  /// What a game second of rain of density one fills, scaled by `density × (0.5 + density)`.
  const FILLING: f32 = 0.012;
  /// What a dry game second drains.
  const DRAINING: f32 = 0.0006;
  /// Game seconds the storm takes to follow the rain by two thirds as it rises, and what a game second takes off it at
  /// most as it eases: the puddles' own draining.
  const STORM_LAG: f32 = 60.0;
  const STORM_EASING: f32 = Self::DRAINING;
  /// What a game second of rain of density one soaks, and what a dry one dries: some hour and a half of a moderate
  /// rain's game time to soak through, twice that to dry.
  const SOAKING: f32 = 0.0004;
  const SOAK_DRYING: f32 = 0.0001;

  /// Moves it on to the real clock `time`, in seconds, under a rain of `density`, its weather running `pace` times as
  /// fast as the game's own clock, and returns how full, how stormy and how soaked.
  pub fn advance(&mut self, time: f32, density: f32, pace: f32) -> (f32, f32, f32) {
    let density: f32 = density.clamp(0.0, 1.0);
    let storminess: f32 = Self::get_storminess(density);
    let (fill, storm, soak): (f32, f32, f32) = match self.state {
      None => (0.0, 0.0, 0.0),
      Some((fill, storm, soak, clock)) => {
        let elapsed: f32 = (time - clock).max(0.0) * pace.max(0.0);
        let change: f32 = if density > 0.0 {
          density * (0.5 + density) * Self::FILLING
        } else {
          -Self::DRAINING
        };
        let followed: f32 = 1.0 - (-elapsed / Self::STORM_LAG).exp();
        let soaking: f32 = if density > 0.0 {
          density * Self::SOAKING
        } else {
          -Self::SOAK_DRYING
        };

        (
          (fill + change * elapsed).clamp(0.0, 1.0),
          if storminess > storm {
            storm + (storminess - storm) * followed
          } else {
            (storm - Self::STORM_EASING * elapsed).max(storminess)
          },
          (soak + soaking * elapsed).clamp(0.0, 1.0),
        )
      }
    };

    self.state = Some((fill, storm, soak, time));

    (fill, storm, soak)
  }

  /// How full, how stormy and how soaked, as last moved on; nothing before.
  pub fn get(&self) -> (f32, f32, f32) {
    self
      .state
      .map_or((0.0, 0.0, 0.0), |(fill, storm, soak, _)| (fill, storm, soak))
  }

  /// How far a rain of a density is past a light one: none to half, all of it at one.
  fn get_storminess(density: f32) -> f32 {
    let past: f32 = ((density - 0.5) / 0.5).clamp(0.0, 1.0);

    past * past * (3.0 - 2.0 * past)
  }
}
