/// A 64-bit FNV-1a hash of bytes, the same on every machine and run, unlike the standard library's hashers: what a
/// cache keys its entries by, so one written in a run is found again in the next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContentHash(u64);

impl ContentHash {
  const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
  const PRIME: u64 = 0x0000_0100_0000_01b3;

  pub const fn new() -> Self {
    Self(Self::OFFSET)
  }

  /// Takes bytes in.
  pub fn write(&mut self, bytes: &[u8]) -> &mut Self {
    for byte in bytes {
      self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(Self::PRIME);
    }

    self
  }

  /// Takes a string in, its length first, so two strings never run into one.
  pub fn write_str(&mut self, text: &str) -> &mut Self {
    self.write(&(text.len() as u64).to_le_bytes()).write(text.as_bytes())
  }

  pub const fn finish(&self) -> u64 {
    self.0
  }
}

impl Default for ContentHash {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::ContentHash;

  #[test]
  fn hashes_bytes_as_fnv_1a() {
    assert_eq!(ContentHash::new().finish(), 0xcbf2_9ce4_8422_2325);
    assert_eq!(ContentHash::new().write(b"a").finish(), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(ContentHash::new().write(b"foobar").finish(), 0x8594_4171_f739_67e8);
  }

  #[test]
  fn keeps_strings_apart() {
    assert_ne!(
      ContentHash::new().write_str("ab").write_str("c").finish(),
      ContentHash::new().write_str("a").write_str("bc").finish()
    );
  }
}
