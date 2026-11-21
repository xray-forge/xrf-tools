use std::io::{Result, Write};

use sha2::{Digest, Sha256};

/// The GUID a build gives content it made: the first 16 bytes of the SHA-256 of everything written to it, so the same
/// input always takes the same identity and a rebuild changes nothing.
///
/// One rule for every derived GUID the tools write (a level's AI map, the game graph, the spawn, a decompiled scene):
/// two copies of it drifting would make builds of the same input disagree. Write the content with its own GUID field
/// nil, as the engine compares that field and not the bytes behind it.
#[derive(Clone, Default)]
pub struct ContentGuid {
  hasher: Sha256,
}

impl ContentGuid {
  pub fn new() -> Self {
    Self::default()
  }

  /// The GUID of `bytes` alone.
  pub fn of(bytes: &[u8]) -> [u8; 16] {
    let mut guid: Self = Self::new();

    guid.update(bytes);
    guid.finish()
  }

  pub fn update(&mut self, bytes: &[u8]) {
    self.hasher.update(bytes);
  }

  /// The 16 bytes of the GUID, in the order the hash gave them.
  pub fn finish(self) -> [u8; 16] {
    let mut guid: [u8; 16] = [0; 16];

    guid.copy_from_slice(&self.hasher.finalize()[..16]);

    guid
  }
}

/// A writer the content writes itself into, as a file would take it.
impl Write for ContentGuid {
  fn write(&mut self, bytes: &[u8]) -> Result<usize> {
    self.update(bytes);

    Ok(bytes.len())
  }

  fn flush(&mut self) -> Result<()> {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use super::ContentGuid;

  #[test]
  fn takes_the_front_of_the_sha256_of_what_it_was_given() {
    // SHA-256 of "abc", the standard's first example: ba7816bf 8f01cfea 414140de 5dae2223 ...
    assert_eq!(
      ContentGuid::of(b"abc"),
      [
        0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE, 0x5D, 0xAE, 0x22, 0x23
      ]
    );
  }

  #[test]
  fn gives_the_same_guid_however_the_content_arrives() {
    let mut written: ContentGuid = ContentGuid::new();

    written.write_all(b"a").unwrap();
    written.update(b"bc");

    assert_eq!(written.finish(), ContentGuid::of(b"abc"));
  }
}
