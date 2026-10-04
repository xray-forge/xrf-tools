/// `CRandom`: the Microsoft C runtime's generator, which `drand48` and `NRand` draw from.
#[derive(Clone, Debug)]
pub struct ParticleRandom {
  hold: i32,
}

impl ParticleRandom {
  /// `CRandom::maxI`, the largest number `next_integer` returns.
  pub const MAX_INTEGER: i32 = 32_767;

  /// `ONE_OVER_SIGMA_EXP`, `NRand`'s scale from its exponential draw to a unit deviation.
  const ONE_OVER_SIGMA_EXP: f32 = 1.0 / 0.7975;

  pub const fn new(seed: i32) -> Self {
    Self { hold: seed }
  }

  /// `randI()`: the next number in `0..=32767`.
  pub fn next_integer(&mut self) -> i32 {
    self.hold = self.hold.wrapping_mul(214_013).wrapping_add(2_531_011);

    (self.hold >> 16) & Self::MAX_INTEGER
  }

  /// `randI(max)`: the next number in `0..max`.
  pub fn next_below(&mut self, max: i32) -> i32 {
    self.next_integer() % max
  }

  /// `randF()`, `drand48()`: the next fraction in `0..=1`.
  pub fn next_fraction(&mut self) -> f32 {
    self.next_integer() as f32 / Self::MAX_INTEGER as f32
  }

  /// `NRand(sigma)`: a normally distributed number by rejection from an exponential draw.
  pub fn next_normal(&mut self, sigma: f32) -> f32 {
    if sigma == 0.0 {
      return 0.0;
    }

    let mut y: f32;

    loop {
      y = -self.next_fraction().ln();

      if self.next_fraction() <= (-(y - 1.0) * (y - 1.0) * 0.5).exp() {
        break;
      }
    }

    if self.next_integer() & 1 != 0 {
      y * sigma * Self::ONE_OVER_SIGMA_EXP
    } else {
      -y * sigma * Self::ONE_OVER_SIGMA_EXP
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ParticleRandom;

  #[test]
  fn draws_the_microsoft_runtime_sequence_from_seed_one() {
    // `srand(1)` then `rand()`: the C runtime's documented first three numbers.
    let mut random: ParticleRandom = ParticleRandom::new(1);

    assert_eq!(random.next_integer(), 41);
    assert_eq!(random.next_integer(), 18_467);
    assert_eq!(random.next_integer(), 6_334);
  }

  #[test]
  fn scales_a_draw_to_a_fraction_of_the_largest() {
    let mut random: ParticleRandom = ParticleRandom::new(1);

    assert_eq!(random.next_fraction(), 41.0 / 32_767.0);
  }

  #[test]
  fn draws_no_deviation_for_a_zero_sigma() {
    let mut random: ParticleRandom = ParticleRandom::new(7);

    assert_eq!(random.next_normal(0.0), 0.0);
    assert_eq!(random.next_integer(), ParticleRandom::new(7).next_integer());
  }
}
