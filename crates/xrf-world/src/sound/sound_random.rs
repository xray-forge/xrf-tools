use std::time::Duration;

use xrf_renderer::WeatherRandom;

/// `CRandom` as the sound schedules draw from it, seeded so a driven run plays the same sounds at the same moments.
pub struct SoundRandom {
  random: WeatherRandom,
}

impl SoundRandom {
  pub fn new(seed: u64) -> Self {
    Self {
      random: WeatherRandom::new(seed),
    }
  }

  /// `Random.randI(count)`: an index below `count`, none for none.
  pub fn index(&mut self, count: usize) -> usize {
    if count == 0 {
      return 0;
    }

    ((self.random.next_fraction() * count as f32) as usize).min(count - 1)
  }

  /// `Random.randI(least, most)` milliseconds: `least` where the range is empty.
  pub fn millis(&mut self, [least, most]: [u32; 2]) -> Duration {
    let span: u32 = most.saturating_sub(least);

    Duration::from_millis(u64::from(least) + (span as f32 * self.random.next_fraction()) as u64)
  }

  /// `Random.randF(least, most)`.
  pub fn between(&mut self, least: f32, most: f32) -> f32 {
    self.random.between(least, most)
  }

  /// `Random.randF()`.
  pub fn fraction(&mut self) -> f32 {
    self.random.next_fraction()
  }
}
