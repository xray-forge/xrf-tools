/// Philox4x64-10, the counter-based generator of Salmon et al., "Parallel random numbers: as easy as 1, 2, 3" (SC11,
/// the Random123 library), which C++26 adopts as `std::philox4x64`: a keyed bijection from a 256-bit counter to 256
/// random bits. The same key and counter give the same bits on every machine, thread and run, and no state is kept
/// between calls.
pub struct Philox4x64;

impl Philox4x64 {
  /// The rounds Random123 and C++26 settle on for 64-bit words.
  const ROUNDS: usize = 10;

  /// The round multipliers.
  const MULTIPLIERS: [u64; 2] = [0xd2e7_470e_e14c_6c93, 0xca5a_8263_9512_1157];

  /// What the key grows by each round, the golden ratio's and `sqrt(3) − 1`'s bits.
  const KEY_STEPS: [u64; 2] = [0x9e37_79b9_7f4a_7c15, 0xbb67_ae85_84ca_a73b];

  /// The four random words of `counter` under `key`.
  pub fn generate(counter: [u64; 4], key: [u64; 2]) -> [u64; 4] {
    let mut words: [u64; 4] = counter;
    let mut key: [u64; 2] = key;

    for round in 0..Self::ROUNDS {
      if round > 0 {
        key = [
          key[0].wrapping_add(Self::KEY_STEPS[0]),
          key[1].wrapping_add(Self::KEY_STEPS[1]),
        ];
      }

      let [high0, low0] = Self::multiply(Self::MULTIPLIERS[0], words[0]);
      let [high1, low1] = Self::multiply(Self::MULTIPLIERS[1], words[2]);

      words = [high1 ^ words[1] ^ key[0], low1, high0 ^ words[3] ^ key[1], low0];
    }

    words
  }

  /// The full product of `a` and `b`, its high word first.
  fn multiply(a: u64, b: u64) -> [u64; 2] {
    let product: u128 = u128::from(a) * u128::from(b);

    [(product >> 64) as u64, product as u64]
  }
}
