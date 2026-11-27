use std::time::Duration;

use xrf_game_config::ConfigReader;

/// How the game's day runs: the time of day a new game starts at and how many game seconds pass in a simulated one,
/// `[alife]`'s `start_time` and `time_factor`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameClock {
  /// Seconds since midnight.
  pub start_time: f32,
  /// Game seconds a simulated second.
  pub factor: f32,
}

impl Default for GameClock {
  /// Call of Pripyat's: 9:00, ten game seconds a second (`gamedata/configs/alife.ltx`).
  fn default() -> Self {
    Self {
      start_time: 9.0 * 60.0 * 60.0,
      factor: 10.0,
    }
  }
}

impl GameClock {
  /// Seconds in a game day.
  pub const DAY: f32 = 24.0 * 60.0 * 60.0;

  /// The clock `[alife]` sets, as `CALifeTimeManager::init` reads it: `start_time` (`hours:minutes:seconds`) and
  /// `time_factor`, each unconditioned, so one missing or not of its kind is an issue and keeps Call of Pripyat's.
  pub fn read(section: &mut ConfigReader<'_, '_>) -> Self {
    let defaults: Self = Self::default();

    Self {
      start_time: section
        .require::<Duration>("start_time")
        .map_or(defaults.start_time, |time| time.as_secs_f32()),
      factor: section.require_or("time_factor", defaults.factor),
    }
  }

  /// Seconds since midnight once `elapsed` game seconds have passed since the start.
  pub fn get_time_of_day(&self, elapsed: f64) -> f32 {
    (f64::from(self.start_time) + elapsed).rem_euclid(f64::from(Self::DAY)) as f32
  }
}
