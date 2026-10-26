use xrf_error::{XrfError, XrfResult};

use crate::bytes::find_bytes;

/// How deep a document's elements nest, measured from its text before any parser reads it.
///
/// Measured by hand because the parser itself recurses once per open element while it tokenizes, so a document nested
/// deep enough overflows the stack inside the parse, before any tree it builds could be measured.
pub(crate) struct XmlNesting;

impl XmlNesting {
  const COMMENT_START: &'static [u8] = b"<!--";
  const COMMENT_END: &'static [u8] = b"-->";
  const CDATA_START: &'static [u8] = b"<![CDATA[";
  const CDATA_END: &'static [u8] = b"]]>";
  const INSTRUCTION_START: &'static [u8] = b"<?";
  const INSTRUCTION_END: &'static [u8] = b"?>";
  const DECLARATION_START: &'static [u8] = b"<!";
  const CLOSING_START: &'static [u8] = b"</";

  /// Fails when `input` opens more than `limit` elements without closing them.
  ///
  /// Comments, CDATA, processing instructions, a DTD and quoted attribute values are skipped whole, so markup written
  /// inside them never counts. Malformed markup is counted as it reads, and left to the parser to refuse.
  ///
  /// # Errors
  ///
  /// Returns a parsing error naming the line where nesting first passes `limit`.
  pub fn assert_within(input: &str, limit: usize) -> XrfResult {
    let bytes: &[u8] = input.as_bytes();
    let mut depth: usize = 0;
    let mut index: usize = 0;

    while let Some(offset) = bytes[index..].iter().position(|byte| *byte == b'<') {
      let start: usize = index + offset;
      let markup: &[u8] = &bytes[start..];

      index = if markup.starts_with(Self::COMMENT_START) {
        Self::skip_past(bytes, start + Self::COMMENT_START.len(), Self::COMMENT_END)
      } else if markup.starts_with(Self::CDATA_START) {
        Self::skip_past(bytes, start + Self::CDATA_START.len(), Self::CDATA_END)
      } else if markup.starts_with(Self::INSTRUCTION_START) {
        Self::skip_past(bytes, start + Self::INSTRUCTION_START.len(), Self::INSTRUCTION_END)
      } else if markup.starts_with(Self::DECLARATION_START) {
        Self::skip_tag(bytes, start + Self::DECLARATION_START.len())
      } else if markup.starts_with(Self::CLOSING_START) {
        depth = depth.saturating_sub(1);

        Self::skip_tag(bytes, start + Self::CLOSING_START.len())
      } else {
        let end: usize = Self::skip_tag(bytes, start + 1);

        if !bytes[..end].ends_with(b"/>") {
          depth += 1;

          if depth > limit {
            return Err(XrfError::new_parsing_error(format!(
              "Failed to parse XML: elements nest deeper than {limit} levels at line {}",
              bytes[..start].iter().filter(|byte| **byte == b'\n').count() + 1
            )));
          }
        }

        end
      };
    }

    Ok(())
  }

  /// The offset just past the first `terminator` at or after `from`, or the end of the input.
  fn skip_past(bytes: &[u8], from: usize, terminator: &[u8]) -> usize {
    bytes
      .get(from..)
      .and_then(|rest| find_bytes(rest, terminator))
      .map_or(bytes.len(), |offset| from + offset + terminator.len())
  }

  /// The offset just past the `>` closing the tag that runs from `from`, or the end of the input.
  ///
  /// Skips a `>` inside quotes, and inside the brackets of a DTD's internal subset, whose declarations and quoted
  /// entity values may hold whole elements.
  fn skip_tag(bytes: &[u8], from: usize) -> usize {
    let mut quote: Option<u8> = None;
    let mut brackets: usize = 0;

    for (index, byte) in bytes.iter().enumerate().skip(from) {
      match quote {
        Some(open) if *byte == open => quote = None,
        Some(_) => {}
        None => match byte {
          b'"' | b'\'' => quote = Some(*byte),
          b'[' => brackets += 1,
          b']' => brackets = brackets.saturating_sub(1),
          b'>' if brackets == 0 => return index + 1,
          _ => {}
        },
      }
    }

    bytes.len()
  }
}

#[cfg(test)]
mod tests {
  use crate::xml_nesting::XmlNesting;

  fn nested(depth: usize) -> String {
    format!("{}x{}", "<a>".repeat(depth), "</a>".repeat(depth))
  }

  #[test]
  fn accepts_nesting_up_to_the_limit() {
    assert!(XmlNesting::assert_within(&nested(4), 4).is_ok());
    assert!(XmlNesting::assert_within("<a><b/><b/><b></b><b x='1'/></a>", 2).is_ok());
  }

  #[test]
  fn refuses_nesting_past_the_limit_naming_its_line() {
    let error: String = XmlNesting::assert_within("<a>\n<b>\n<c>", 2).unwrap_err().to_string();

    assert!(error.contains("deeper than 2 levels at line 3"), "{error}");
  }

  #[test]
  fn counts_no_markup_inside_comments_cdata_instructions_quotes_or_a_dtd() {
    for source in [
      "<a><!-- <b><b><b> --></a>",
      "<a><![CDATA[<b><b><b>]]></a>",
      "<a><?pi <b><b><b>?></a>",
      "<a note=\"<b>\" other='>'><x/></a>",
      "<!DOCTYPE a [<!ENTITY e '<b><b><b>'><!ELEMENT a ANY>]><a>&e;</a>",
    ] {
      assert!(XmlNesting::assert_within(source, 1).is_ok(), "for {source}");
    }
  }

  #[test]
  fn counts_unclosed_elements_of_a_malformed_document() {
    assert!(XmlNesting::assert_within(&"<a>".repeat(100_000), 32).is_err());
  }
}
