use crate::syntax::LTX_SYMBOL_INCLUDE_WILDCARD;

/// How a wildcard `#include` such as `w_*.ltx` picks the files it names, the same for every source.
pub(crate) struct LtxIncludeMask;

impl LtxIncludeMask {
  /// Whether a statement is a mask over a directory rather than one file's name.
  pub fn is_mask(statement: &str) -> bool {
    statement.contains(LTX_SYMBOL_INCLUDE_WILDCARD)
  }

  /// Whether a file name matches a `*` mask, both as encoded bytes.
  ///
  /// Bytes rather than `str` because a filename is not required to be valid Unicode: a Unix name is bytes, and such a
  /// file is a config the engine would load. Both encodings are self-synchronizing supersets of UTF-8, so searching a
  /// mask part cannot match inside a multi-byte sequence.
  pub fn matches(file_name: &[u8], mask: &[u8]) -> bool {
    let wildcard: u8 = LTX_SYMBOL_INCLUDE_WILDCARD as u8;
    let mut remaining: &[u8] = file_name;
    let mut is_first_part: bool = true;

    for part in mask.split(|byte| *byte == wildcard).filter(|part| !part.is_empty()) {
      let Some(position) = Self::find(remaining, part) else {
        return false;
      };

      if is_first_part && mask.first() != Some(&wildcard) && position != 0 {
        return false;
      }

      remaining = &remaining[position + part.len()..];
      is_first_part = false;
    }

    mask.last() == Some(&wildcard) || remaining.is_empty()
  }

  /// Offset of the first occurrence of `needle`, which is never empty here.
  fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
  }
}

#[cfg(test)]
mod tests {
  use crate::source::LtxIncludeMask;

  #[test]
  fn matches_a_mask_at_either_end_and_in_the_middle() {
    assert!(LtxIncludeMask::matches(b"w_ak74.ltx", b"w_*.ltx"));
    assert!(LtxIncludeMask::matches(b"w_.ltx", b"w_*.ltx"));
    assert!(LtxIncludeMask::matches(b"anything.ltx", b"*.ltx"));
    assert!(LtxIncludeMask::matches(b"w_ak74.ltx", b"w_*"));
    assert!(!LtxIncludeMask::matches(b"aw_ak74.ltx", b"w_*.ltx"));
    assert!(!LtxIncludeMask::matches(b"w_ak74.ltx.bak", b"w_*.ltx"));
  }

  /// The mask comparison is over bytes, so a name that does not decode is still weighed against the mask instead of being
  /// dropped. This runs everywhere; reading such a file needs a Unix filesystem, because Windows cannot hold the name.
  #[test]
  fn matches_a_mask_over_bytes_that_are_not_valid_unicode() {
    // 0xFF cannot appear in UTF-8, so this name is only reachable as bytes.
    let name: &[u8] = b"section_\xffbroken.ltx";

    assert!(LtxIncludeMask::matches(name, b"section_*.ltx"));
    assert!(LtxIncludeMask::matches(name, b"*broken*"));
    assert!(!LtxIncludeMask::matches(name, b"weapon_*.ltx"));
    assert!(!LtxIncludeMask::matches(name, b"section_*.xml"));
  }

  #[test]
  fn tells_a_mask_from_a_name() {
    assert!(LtxIncludeMask::is_mask("items\\w_*.ltx"));
    assert!(!LtxIncludeMask::is_mask("items\\w_ak74.ltx"));
  }
}
