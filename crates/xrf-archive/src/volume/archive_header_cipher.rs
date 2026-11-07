/// The cipher Shadow of Chornobyl's volumes hide their entry name table under, `trivial_encryptor`
/// (`xrCore/Crypto/trivial_encryptor.cpp`): each byte substituted through a shuffled 256-entry table and masked with a
/// seeded random stream. `CLocatorAPI::LoadArchive` applies it to a volume carrying no `[header]` metadata, trying the
/// worldwide key and then the Russian one; the payloads themselves are not enciphered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchiveHeaderCipher {
  Worldwide,
  Russian,
}

impl ArchiveHeaderCipher {
  /// Every key, in the order the engine tries them.
  pub(crate) const ALL: [Self; 2] = [Self::Worldwide, Self::Russian];

  /// The key's table shuffle count, table seed and stream seed, `m_key_worldwide` and `m_key_russian`.
  const fn get_key(self) -> (u32, u32, u32) {
    match self {
      Self::Worldwide => (1024, 6_011_979, 24_031_979),
      Self::Russian => (2048, 20_091_958, 20_031_955),
    }
  }

  /// The substitution table, `m_alphabet`: the identity shuffled by swapping two distinct random entries the key's
  /// number of times.
  fn get_alphabet(self) -> [u8; 256] {
    let (iterations, table_seed, _) = self.get_key();
    let mut alphabet: [u8; 256] = std::array::from_fn(|index| index as u8);
    let mut table: Random32 = Random32(table_seed);

    for _ in 0..iterations {
      let first: usize = table.next_byte_index();
      let mut second: usize = table.next_byte_index();

      while first == second {
        second = table.next_byte_index();
      }

      alphabet.swap(first, second);
    }

    alphabet
  }

  /// The random stream each byte is masked with, from its start.
  fn get_stream(self) -> Random32 {
    Random32(self.get_key().2)
  }

  /// `bytes` deciphered, `trivial_encryptor::decode`.
  pub(crate) fn decode(self, bytes: &[u8]) -> Vec<u8> {
    let alphabet: [u8; 256] = self.get_alphabet();
    let mut back: [u8; 256] = [0; 256];

    for (index, value) in alphabet.iter().enumerate() {
      back[usize::from(*value)] = index as u8;
    }

    let mut stream: Random32 = self.get_stream();

    bytes
      .iter()
      .map(|byte| back[usize::from(byte ^ stream.next_byte_index() as u8)])
      .collect()
  }
}

#[cfg(test)]
impl ArchiveHeaderCipher {
  /// `trivial_encryptor::encode`, which the engine's packer used and nothing here writes: the substitution, then the
  /// mask.
  pub(crate) fn encode(self, bytes: &[u8]) -> Vec<u8> {
    let alphabet: [u8; 256] = self.get_alphabet();
    let mut stream: Random32 = self.get_stream();

    bytes
      .iter()
      .map(|byte| alphabet[usize::from(*byte)] ^ stream.next_byte_index() as u8)
      .collect()
  }
}

/// `CRandom32`: a linear congruential generator, its draw scaled into a range by the high half of a 64-bit product.
struct Random32(u32);

impl Random32 {
  /// A draw in `0..256`.
  fn next_byte_index(&mut self) -> usize {
    self.0 = self.0.wrapping_mul(0x0808_8405).wrapping_add(1);

    ((u64::from(self.0) * 256) >> 32) as usize
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveHeaderCipher;

  #[test]
  fn deciphers_what_each_key_enciphered() {
    let plain: Vec<u8> = (0..=255)
      .chain(b"meshes\\actors\\stalker_bandit_1.ogf".iter().copied())
      .collect();

    for cipher in ArchiveHeaderCipher::ALL {
      let enciphered: Vec<u8> = cipher.encode(&plain);

      assert_ne!(enciphered, plain);
      assert_eq!(cipher.decode(&enciphered), plain);
    }

    assert_ne!(
      ArchiveHeaderCipher::Worldwide.encode(&plain),
      ArchiveHeaderCipher::Russian.encode(&plain)
    );
  }

  #[test]
  fn shuffles_a_permutation() {
    for cipher in ArchiveHeaderCipher::ALL {
      let mut alphabet: Vec<u8> = cipher.get_alphabet().to_vec();

      alphabet.sort_unstable();

      assert_eq!(alphabet, (0..=255).collect::<Vec<u8>>());
    }
  }
}
