use crate::philox::philox4x64::Philox4x64;
use crate::stream::random_address::RandomAddress;

/// The random numbers at one address under one seed, drawn in order: Philox4x64-10 over the counter `[draw block,
/// domain, entity, step]` keyed by `[seed, 0]`. Nothing is kept but where it stands, so a stream is made where it is
/// needed and dropped after, and a save keeps only the seed. At the zero address it is C++26's `std::philox4x64`
/// seeded with the seed.
///
/// Draws are integer arithmetic, and a float is a scaled integer, exact on every machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RandomStream {
  key: [u64; 2],
  address: RandomAddress,
  /// The block of four words drawn from now, and the next word of it.
  block: u64,
  words: [u64; 4],
  next: usize,
}

impl RandomStream {
  /// The stream at `address` under `seed`.
  pub fn new(seed: u64, address: RandomAddress) -> Self {
    let key: [u64; 2] = [seed, 0];

    Self {
      key,
      address,
      block: 0,
      words: Philox4x64::generate([0, address.domain, address.entity, address.step], key),
      next: 0,
    }
  }

  /// The next 64 random bits.
  pub fn next_u64(&mut self) -> u64 {
    if self.next == self.words.len() {
      self.block = self.block.wrapping_add(1);
      self.words = Philox4x64::generate(
        [self.block, self.address.domain, self.address.entity, self.address.step],
        self.key,
      );
      self.next = 0;
    }

    let word: u64 = self.words[self.next];

    self.next += 1;

    word
  }

  /// The next 32 random bits, the high half of a word.
  pub fn next_u32(&mut self) -> u32 {
    (self.next_u64() >> 32) as u32
  }

  /// A number from 0 up to 1, never 1, on a grid of 2⁻²⁴: every value an `f32` can hold evenly there.
  pub fn next_f32(&mut self) -> f32 {
    (self.next_u64() >> 40) as f32 * (1.0 / (1_u32 << 24) as f32)
  }

  /// A number from 0 up to 1, never 1, on a grid of 2⁻⁵³.
  pub fn next_f64(&mut self) -> f64 {
    (self.next_u64() >> 11) as f64 * (1.0 / (1_u64 << 53) as f64)
  }

  /// A whole number from 0 up to `bound`, never `bound`, every one as likely: Lemire's multiply-and-reject ("Fast
  /// random integer generation in an interval", 2019). Zero for a bound of zero.
  pub fn below(&mut self, bound: u64) -> u64 {
    if bound == 0 {
      return 0;
    }

    // Products whose low word falls under `2^64 mod bound` would favour some results; they are drawn again.
    let threshold: u64 = bound.wrapping_neg() % bound;

    loop {
      let product: u128 = u128::from(self.next_u64()) * u128::from(bound);

      if (product as u64) >= threshold {
        return (product >> 64) as u64;
      }
    }
  }

  /// True with probability `chance`, from 0, never, to 1, always.
  pub fn chance(&mut self, chance: f32) -> bool {
    self.next_f32() < chance
  }
}
