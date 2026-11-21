use lzokay::compress::Dict;
use xrf_error::{XrfError, XrfResult};

/// Compresses entry payloads the way a volume stores them, LZO1X as xrCompress writes and the engine reads.
///
/// Keeps the compressor's working dictionary between entries, so one per worker compresses a whole volume without
/// allocating it again for each.
pub struct ArchiveCompressor {
  dict: Box<Dict>,
}

impl ArchiveCompressor {
  pub fn new() -> Self {
    Self { dict: Dict::new() }
  }

  /// `contents` compressed, for the entry named `name` in the error.
  ///
  /// # Errors
  ///
  /// Returns an unexpected error when the compressor fails.
  pub fn compress(&mut self, contents: &[u8], name: &str) -> XrfResult<Vec<u8>> {
    lzokay::compress::compress_with_dict(contents, &mut self.dict)
      .map_err(|error| XrfError::new_unexpected_error(format!("Failed to compress '{name}': {error}")))
  }
}

impl Default for ArchiveCompressor {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveCompressor;

  #[test]
  fn compresses_each_entry_so_the_reader_restores_it() {
    let mut compressor: ArchiveCompressor = ArchiveCompressor::new();
    let entries: [&[u8]; 2] = [&b"[section]\nkey = value\n".repeat(64), b"short"];

    // One compressor across entries, as a packing worker uses it, so the kept dictionary never leaks between them.
    for contents in entries {
      let compressed: Vec<u8> = compressor.compress(contents, "configs\\system.ltx").unwrap();
      let mut restored: Vec<u8> = vec![0; contents.len()];
      let written: usize = lzokay::decompress::decompress(&compressed, &mut restored).unwrap();

      assert_eq!(&restored[..written], contents);
    }
  }
}
