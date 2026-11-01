/// `Random.randF`: numbers in `[0, 1)` from a seed, the same run for the same seed.
#[derive(Clone, Debug)]
pub struct WeatherRandom {
  state: u64,
}

impl WeatherRandom {
  pub fn new(seed: u64) -> Self {
    Self { state: seed.max(1) }
  }

  /// The next number, `xorshift64*`'s top 24 bits as a fraction.
  pub fn next_fraction(&mut self) -> f32 {
    self.state ^= self.state >> 12;
    self.state ^= self.state << 25;
    self.state ^= self.state >> 27;

    (self.state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32 / (1u64 << 24) as f32
  }

  /// `Random.randF(min, max)`.
  pub fn between(&mut self, min: f32, max: f32) -> f32 {
    min + (max - min) * self.next_fraction()
  }
}
