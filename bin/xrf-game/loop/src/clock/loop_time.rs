use std::time::Duration;

use crate::clock::game_clock::GameClock;

/// Where the simulation's time stands as a tick is stepped: what every system reads of time, never the wall clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopTime {
  /// The tick being stepped from; the step makes it the next.
  pub tick: u64,
  /// The simulated time a step covers.
  pub step: Duration,
  pub game_clock: GameClock,
  /// Game seconds since the game started.
  pub elapsed: f64,
}

impl LoopTime {
  /// Time at tick 0, stepping by `step`, the day run by `game_clock`.
  pub fn new(step: Duration, game_clock: GameClock) -> Self {
    Self {
      tick: 0,
      step,
      game_clock,
      elapsed: 0.0,
    }
  }

  /// Game seconds a step covers.
  pub fn get_game_step(&self) -> f64 {
    self.step.as_secs_f64() * f64::from(self.game_clock.factor)
  }

  /// Game seconds since midnight.
  pub fn get_time_of_day(&self) -> f32 {
    self.game_clock.get_time_of_day(self.elapsed)
  }

  /// The time a step later.
  pub(crate) fn next(&self) -> Self {
    Self {
      tick: self.tick + 1,
      elapsed: self.elapsed + self.get_game_step(),
      ..*self
    }
  }
}
