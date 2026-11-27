use std::time::Duration;

use crate::time::game_clock::GameClock;

/// The game's own time: what a save keeps and what the game sets (a sleep, a loaded save, a changed time factor), kept
/// in the game's state rather than in the loop, which counts only ticks. Whole milliseconds of game time since the game
/// began, as X-Ray keeps its game time, and the clock its day runs by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameTime {
  clock: GameClock,
  /// Game milliseconds since the game began.
  elapsed: u64,
}

impl GameTime {
  /// A game's time as it begins, its day run by `clock`.
  pub fn new(clock: GameClock) -> Self {
    Self { clock, elapsed: 0 }
  }

  pub fn get_clock(&self) -> &GameClock {
    &self.clock
  }

  /// Game milliseconds since the game began.
  pub fn get_elapsed(&self) -> u64 {
    self.elapsed
  }

  /// Game seconds since midnight.
  pub fn get_time_of_day(&self) -> f32 {
    self.clock.get_time_of_day(self.elapsed as f64 / 1000.0)
  }

  /// Moves on by the game time a simulated `step` covers at the clock's factor, to the nearest millisecond: exact at
  /// Call of Pripyat's 10 ms ticks and ten game seconds a second.
  pub fn advance(&mut self, step: Duration) {
    let game: f64 = step.as_secs_f64() * f64::from(self.clock.factor) * 1000.0;

    self.elapsed = self.elapsed.saturating_add(game.round() as u64);
  }
}
