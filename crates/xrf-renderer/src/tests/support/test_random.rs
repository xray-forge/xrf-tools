/// A linear congruential generator, so a test's random sequence is the same every run.
pub(crate) struct TestRandom {
  seed: u64,
}

impl TestRandom {
  pub(crate) fn new(seed: u64) -> Self {
    Self { seed }
  }

  /// The next number below `bound`.
  pub(crate) fn next(&mut self, bound: u64) -> u64 {
    self.seed = self
      .seed
      .wrapping_mul(6_364_136_223_846_793_005)
      .wrapping_add(1_442_695_040_888_963_407);

    (self.seed >> 33) % bound
  }
}
