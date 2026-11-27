/// Where a stream of random numbers sits, beside the seed: what it is for, for which thing, at which step. Two streams
/// differing in any part are unrelated, so whoever draws, in whatever order and on whatever thread, draws the same.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RandomAddress {
  /// What the numbers decide, one per kind of decision ([`RandomAddress::domain`] names one).
  pub domain: u64,
  /// The thing deciding: an object's id, a cell, none.
  pub entity: u64,
  /// When: a simulation's tick.
  pub step: u64,
}

impl RandomAddress {
  /// The domain `name` stands for: its 64-bit FNV-1a hash, so a domain is named where it is used, not numbered in a
  /// list every user must share; two names meet by chance about once in four billion pairs.
  pub const fn domain(name: &str) -> u64 {
    let bytes: &[u8] = name.as_bytes();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut index: usize = 0;

    while index < bytes.len() {
      hash ^= bytes[index] as u64;
      hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
      index += 1;
    }

    hash
  }
}
