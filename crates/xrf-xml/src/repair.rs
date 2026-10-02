/// Rewrite the constructs X-Ray accepts and XML does not, without moving a single byte.
///
/// Every substitution replaces one byte with one byte, so offsets into the result are offsets into
/// the input. That is the whole point: the repaired copy is only ever parsed, and every range it
/// yields still addresses the original text, so an edit splices what was actually on disk.
pub(crate) fn repair_for_parsing(input: &str) -> String {
  let mut repaired: Vec<u8> = input.as_bytes().to_vec();
  let bytes: &[u8] = input.as_bytes();
  let mut index: usize = 0;

  while index < bytes.len() {
    if bytes[index..].starts_with(b"<!--") {
      let body: usize = index + 4;
      let end: usize = find(bytes, body, b"-->").unwrap_or(bytes.len());

      // A comment body may not contain `--`, and shipped banners are made of little else. Blanking
      // every dash in the body is same-length and cannot break the terminator, which sits outside it.
      for byte in &mut repaired[body..end] {
        if *byte == b'-' {
          *byte = b'~';
        }
      }

      index = end.saturating_add(3).min(bytes.len());

      continue;
    }

    // A declaration has to open the document, and the engine's reader skips one anywhere else: shipped mods write
    // banners above it. Blanking it with spaces keeps its line breaks, so positions still read the same.
    if index > 0 && is_declaration_at(bytes, index) {
      let end: usize = find(bytes, index, b"?>").map_or(bytes.len(), |end| end + 2);

      for byte in &mut repaired[index..end] {
        if !byte.is_ascii_whitespace() {
          *byte = b' ';
        }
      }

      index = end;

      continue;
    }

    // A bare ampersand is not a reference, and translation text is full of them.
    if bytes[index] == b'&' && !is_reference_at(bytes, index) {
      repaired[index] = b'~';
    }

    index += 1;
  }

  String::from_utf8(repaired).unwrap_or_else(|_| input.to_owned())
}

/// Whether an XML declaration starts at `start`, rather than a processing instruction such as `<?xml-stylesheet`.
fn is_declaration_at(bytes: &[u8], start: usize) -> bool {
  bytes[start..].starts_with(b"<?xml") && bytes.get(start + 5).is_some_and(u8::is_ascii_whitespace)
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

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
  bytes
    .get(from..)?
    .windows(needle.len())
    .position(|window| window == needle)
    .map(|offset| from + offset)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_repair_keeps_the_input_length() {
    // The one invariant everything else depends on: same length means offsets still line up.
    for source in [
      "<!-- ---- names ---- -->",
      "<text>Smith & Wesson</text>",
      "<text>a &amp; b</text>",
      "<!-- banner -->\r\n<?xml version='1.0' encoding=\"windows-1251\"?>\r\n<root/>",
      "<root/>",
    ] {
      assert_eq!(repair_for_parsing(source).len(), source.len(), "for {source}");
    }
  }

  #[test]
  fn blanks_dashes_inside_a_comment_body() {
    assert_eq!(repair_for_parsing("<!-- -- -->"), "<!-- ~~ -->");
  }

  #[test]
  fn leaves_the_comment_terminator_intact() {
    // Eating the closing dashes would leave the comment unterminated and unparseable.
    assert!(repair_for_parsing("<!------->").ends_with("-->"));
  }

  #[test]
  fn neutralizes_a_bare_ampersand() {
    assert_eq!(repair_for_parsing("a & b"), "a ~ b");
  }

  #[test]
  fn leaves_a_real_entity_alone() {
    assert_eq!(repair_for_parsing("a &amp; b"), "a &amp; b");
    assert_eq!(repair_for_parsing("a &#38; b"), "a &#38; b");
  }

  #[test]
  fn treats_an_unterminated_entity_as_a_bare_ampersand() {
    assert_eq!(repair_for_parsing("a &amp b"), "a ~amp b");
  }

  #[test]
  fn blanks_a_declaration_that_does_not_open_the_document() {
    assert_eq!(
      repair_for_parsing("<!-- banner -->\n<?xml version=\"1.0\"?>\n<root/>"),
      "<!-- banner -->\n                     \n<root/>"
    );
  }

  #[test]
  fn leaves_an_opening_declaration_and_other_processing_instructions_alone() {
    for source in [
      "<?xml version=\"1.0\"?><root/>",
      "<root><?xml-stylesheet href=\"a\"?></root>",
    ] {
      assert_eq!(repair_for_parsing(source), source);
    }
  }

  #[test]
  fn leaves_a_well_formed_document_untouched() {
    let source: &str = "<root><child id=\"a\">text</child></root>";

    assert_eq!(repair_for_parsing(source), source);
  }
}
