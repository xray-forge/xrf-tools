use crate::bytes::find_bytes;

/// Rewrites the constructs X-Ray accepts and XML does not, without moving a single byte.
///
/// Every substitution replaces one byte with one byte, so offsets into the result are offsets into
/// the input. That is the whole point: the repaired copy is only ever parsed, and every range it
/// yields still addresses the original text, so an edit splices what was actually on disk.
pub(crate) struct XmlRepair;

impl XmlRepair {
  const COMMENT_START: &'static [u8] = b"<!--";
  const COMMENT_END: &'static [u8] = b"-->";
  const DECLARATION_START: &'static [u8] = b"<?xml";
  const DECLARATION_END: &'static [u8] = b"?>";

  /// A same-length copy of `input` with comment banners, bare ampersands and a misplaced declaration neutralized.
  pub fn repair(input: &str) -> String {
    let bytes: &[u8] = input.as_bytes();
    let mut repaired: Vec<u8> = bytes.to_vec();
    let mut index: usize = 0;

    while index < bytes.len() {
      if bytes[index..].starts_with(Self::COMMENT_START) {
        let body: usize = index + Self::COMMENT_START.len();
        let end: usize = Self::find_from(bytes, body, Self::COMMENT_END).unwrap_or(bytes.len());

        // A comment body may not contain `--`, and shipped banners are made of little else. Blanking
        // every dash in the body is same-length and cannot break the terminator, which sits outside it.
        for byte in &mut repaired[body..end] {
          if *byte == b'-' {
            *byte = b'~';
          }
        }

        index = end.saturating_add(Self::COMMENT_END.len()).min(bytes.len());

        continue;
      }

      // A declaration has to open the document, and the engine's reader skips one anywhere else: shipped mods write
      // banners above it. Blanking it with spaces keeps its line breaks, so positions still read the same.
      if index > 0 && Self::is_declaration_at(bytes, index) {
        let end: usize = Self::find_from(bytes, index, Self::DECLARATION_END)
          .map_or(bytes.len(), |end| end + Self::DECLARATION_END.len());

        for byte in &mut repaired[index..end] {
          if !byte.is_ascii_whitespace() {
            *byte = b' ';
          }
        }

        index = end;

        continue;
      }

      // A bare ampersand is not a reference, and translation text is full of them.
      if bytes[index] == b'&' && !Self::is_reference_at(bytes, index) {
        repaired[index] = b'~';
      }

      index += 1;
    }

    // Every substitution writes an ASCII byte, which cannot leave a multi-byte character half replaced.
    String::from_utf8(repaired).expect("Repairing XML writes only ASCII bytes")
  }

  /// Whether an XML declaration starts at `start`, rather than a processing instruction such as `<?xml-stylesheet`.
  fn is_declaration_at(bytes: &[u8], start: usize) -> bool {
    bytes[start..].starts_with(Self::DECLARATION_START)
      && bytes
        .get(start + Self::DECLARATION_START.len())
        .is_some_and(u8::is_ascii_whitespace)
  }

  /// Whether the ampersand at `start` begins something shaped like an entity reference.
  fn is_reference_at(bytes: &[u8], start: usize) -> bool {
    let mut index: usize = start + 1;

    if bytes.get(index) == Some(&b'#') {
      index += 1;
    }

    let name_start: usize = index;

    while let Some(byte) = bytes.get(index) {
      match byte {
        b';' => return index > name_start,
        byte if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-') => index += 1,
        _ => return false,
      }
    }

    false
  }

  fn find_from(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    find_bytes(bytes.get(from..)?, needle).map(|offset| from + offset)
  }
}

#[cfg(test)]
mod tests {
  use crate::source::xml_repair::XmlRepair;

  #[test]
  fn every_repair_keeps_the_input_length() {
    // The one invariant everything else depends on: same length means offsets still line up.
    for source in [
      "<!-- ---- names ---- -->",
      "<text>Smith & Wesson</text>",
      "<text>a &amp; b</text>",
      "<!-- banner -->\r\n<?xml version='1.0' encoding=\"windows-1251\"?>\r\n<root/>",
      "<!-- банер -->\n<?xml version=\"1.0\" кодування",
      "<root/>",
    ] {
      assert_eq!(XmlRepair::repair(source).len(), source.len(), "for {source}");
    }
  }

  #[test]
  fn blanks_dashes_inside_a_comment_body() {
    assert_eq!(XmlRepair::repair("<!-- -- -->"), "<!-- ~~ -->");
  }

  #[test]
  fn leaves_the_comment_terminator_intact() {
    // Eating the closing dashes would leave the comment unterminated and unparseable.
    assert!(XmlRepair::repair("<!------->").ends_with("-->"));
  }

  #[test]
  fn neutralizes_a_bare_ampersand() {
    assert_eq!(XmlRepair::repair("a & b"), "a ~ b");
  }

  #[test]
  fn leaves_a_real_entity_alone() {
    assert_eq!(XmlRepair::repair("a &amp; b"), "a &amp; b");
    assert_eq!(XmlRepair::repair("a &#38; b"), "a &#38; b");
  }

  #[test]
  fn treats_an_unterminated_entity_as_a_bare_ampersand() {
    assert_eq!(XmlRepair::repair("a &amp b"), "a ~amp b");
  }

  #[test]
  fn blanks_a_declaration_that_does_not_open_the_document() {
    assert_eq!(
      XmlRepair::repair("<!-- banner -->\n<?xml version=\"1.0\"?>\n<root/>"),
      "<!-- banner -->\n                     \n<root/>"
    );
  }

  #[test]
  fn leaves_an_opening_declaration_and_other_processing_instructions_alone() {
    for source in [
      "<?xml version=\"1.0\"?><root/>",
      "<root><?xml-stylesheet href=\"a\"?></root>",
    ] {
      assert_eq!(XmlRepair::repair(source), source);
    }
  }

  #[test]
  fn leaves_a_well_formed_document_untouched() {
    let source: &str = "<root><child id=\"a\">text</child></root>";

    assert_eq!(XmlRepair::repair(source), source);
  }
}
