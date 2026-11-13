/// The random stream xrAI's spawn build draws from: the MSVC runtime's `rand` and VS2008's `std::random_shuffle`, from
/// the seed every thread starts at, 1. Nothing seeds it, so the same draws in the same order give the shipped result.
pub struct SpawnRandom {
  state: u32,
}

impl SpawnRandom {
  /// The bits `rand` gives, `RAND_MAX`.
  const BITS: u32 = 15;
  const MAX: u64 = (1 << Self::BITS) - 1;

  /// The stream a thread starts with.
  pub fn new() -> Self {
    Self { state: 1 }
  }

  /// `rand`: `holdrand * 214013 + 2531011`, bits 16 to 30.
  pub fn draw(&mut self) -> u32 {
    self.state = self.state.wrapping_mul(214_013).wrapping_add(2_531_011);

    (self.state >> 16) & Self::MAX as u32
  }

  /// `_Random_shuffle`: each element from the second swapped with one at an index drawn below its own plus one, the
  /// draw widened 15 bits at a time, in 32 bits, while the range is smaller.
  pub fn shuffle<T>(&mut self, items: &mut [T]) {
    for index in 2..=items.len() as u64 {
      let (mut range, mut value): (u64, u64) = (Self::MAX, u64::from(self.draw()));

      while range < index && range != u64::from(u32::MAX) {
        range = ((range << Self::BITS) | Self::MAX) & u64::from(u32::MAX);
        value = ((value << Self::BITS) | u64::from(self.draw())) & u64::from(u32::MAX);
      }

      items.swap(index as usize - 1, (value % index) as usize);
    }
  }
}

impl Default for SpawnRandom {
  fn default() -> Self {
    Self::new()
  }
}
