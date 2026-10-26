use xrf_error::{XrfError, XrfResult};
use xrf_utils::{
  XRayEncoding, decode_bytes_to_string, new_utf8_encoder, new_windows1250_encoder, new_windows1251_encoder,
  new_windows1252_encoder,
};

use crate::bytes::find_bytes;

/// The encoding an XML document's bytes are written in, read from its declaration before the bytes are text.
pub struct XmlEncoding;

impl XmlEncoding {
  const UTF8_BYTE_ORDER_MARK: &'static [u8] = &[0xEF, 0xBB, 0xBF];
  const COMMENT_START: &'static [u8] = b"<!--";
  const COMMENT_END: &'static [u8] = b"-->";
  const DECLARATION_START: &'static [u8] = b"<?xml";
  const DECLARATION_END: &'static [u8] = b"?>";

  /// The encoding a document's XML declaration names, if it has one.
  ///
  /// The declaration is looked for where it may stand: after an optional UTF-8 byte order mark, blanks and the comment
  /// banners shipped mods write above it, and before anything else. A declaration quoted inside a comment, a processing
  /// instruction such as `<?xml-stylesheet`, or a string deep in the document therefore never decides how it is read.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when the declaration names a code page there is no encoder for.
  pub fn find_declared(input: &[u8]) -> XrfResult<Option<XRayEncoding>> {
    let Some(label) = Self::find_declaration(input).and_then(Self::find_label) else {
      return Ok(None);
    };

    Ok(Some(Self::from_label(&label)?))
  }

  /// Resolve an encoding name to the encoder that reads and writes it.
  ///
  /// # Errors
  ///
  /// Returns an encoding error for a name none of the X-Ray encoders cover.
  pub fn from_label(label: &str) -> XrfResult<XRayEncoding> {
    let normalized: String = label
      .chars()
      .filter(|character| character.is_ascii_alphanumeric())
      .flat_map(char::to_lowercase)
      .collect();

    match normalized.as_str() {
      "utf8" => Ok(new_utf8_encoder()),
      "cp1250" | "windows1250" => Ok(new_windows1250_encoder()),
      "cp1251" | "windows1251" => Ok(new_windows1251_encoder()),
      "cp1252" | "windows1252" => Ok(new_windows1252_encoder()),
      _ => Err(XrfError::new_encoding_error(format!(
        "Unsupported XML encoding '{label}'"
      ))),
    }
  }

  /// Decode XML bytes from their declaration, defaulting to UTF-8 when no declaration is present.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when the declaration is unsupported or the bytes do not decode.
  pub(crate) fn decode(input: &[u8]) -> XrfResult<String> {
    // Mapped rather than converted: bytes that do not decode are the document's own defect, and `From<io::Error>` would
    // report them as an IO failure, which a caller reads as the file never having been read.
    decode_bytes_to_string(input, Self::find_declared(input)?.unwrap_or_else(new_utf8_encoder))
      .map_err(|error| XrfError::new_encoding_error(error.to_string()))
  }

  /// The body of the declaration that opens the document, between `<?xml` and `?>`.
  ///
  /// Read by hand rather than parsed, because the declaration has to be read before the document can be decoded, and
  /// decoding is what the declaration decides.
  fn find_declaration(input: &[u8]) -> Option<&[u8]> {
    let mut rest: &[u8] = input.strip_prefix(Self::UTF8_BYTE_ORDER_MARK).unwrap_or(input);

    loop {
      rest = rest.trim_ascii_start();

      let Some(comment) = rest.strip_prefix(Self::COMMENT_START) else {
        break;
      };

      rest = &comment[find_bytes(comment, Self::COMMENT_END)? + Self::COMMENT_END.len()..];
    }

    // Matched in any case: this scan only decides the encoding, and judging the declaration is the parser's job.
    let start: &[u8] = rest.get(..Self::DECLARATION_START.len())?;
    let body: &[u8] = &rest[Self::DECLARATION_START.len()..];

    if !start.eq_ignore_ascii_case(Self::DECLARATION_START) || !body.first().is_some_and(u8::is_ascii_whitespace) {
      return None;
    }

    Some(&body[..find_bytes(body, Self::DECLARATION_END)?])
  }

  /// The value of a declaration's `encoding` pseudo-attribute, without judging whether it is supported.
  fn find_label(declaration: &[u8]) -> Option<String> {
    let declaration: String = String::from_utf8_lossy(declaration).into_owned();
    let mut rest: &str = &declaration;

    loop {
      let (name, value) = rest.split_once('=')?;
      let value: &str = value.trim_start();
      let quote: char = value.chars().next().filter(|quote| matches!(quote, '"' | '\''))?;
      let (value, after) = value[quote.len_utf8()..].split_once(quote)?;

      if name.trim().eq_ignore_ascii_case("encoding") {
        return Some(value.to_owned());
      }

      rest = after;
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_utils::{new_utf8_encoder, new_windows1250_encoder, new_windows1251_encoder, new_windows1252_encoder};

  use crate::decode::XmlEncoding;

  #[test]
  fn reads_a_declared_encoding() {
    let input: &[u8] = b"<?xml version=\"1.0\" encoding=\"windows-1251\"?><root/>";

    assert_eq!(
      XmlEncoding::find_declared(input).unwrap(),
      Some(new_windows1251_encoder())
    );
  }

  #[test]
  fn accepts_single_quotes_and_odd_spacing() {
    let input: &[u8] = b"<?xml version='1.0' encoding = 'cp1250' ?><root/>";

    assert_eq!(
      XmlEncoding::find_declared(input).unwrap(),
      Some(new_windows1250_encoder())
    );
  }

  #[test]
  fn reports_nothing_when_there_is_no_declaration() {
    assert_eq!(XmlEncoding::find_declared(b"<root/>").unwrap(), None);
    assert_eq!(
      XmlEncoding::find_declared(b"<?xml version=\"1.0\"?><root/>").unwrap(),
      None
    );
  }

  #[test]
  fn refuses_an_encoding_with_no_encoder() {
    let error = XmlEncoding::find_declared(b"<?xml version=\"1.0\" encoding=\"shift_jis\"?><root/>").unwrap_err();

    assert!(error.to_string().contains("Unsupported XML encoding 'shift_jis'"));
  }

  #[test]
  fn normalizes_punctuation_and_case_in_a_label() {
    assert_eq!(
      XmlEncoding::from_label("WINDOWS-1252").unwrap(),
      new_windows1252_encoder()
    );
    assert_eq!(XmlEncoding::from_label("cp_1251").unwrap(), new_windows1251_encoder());
    assert_eq!(XmlEncoding::from_label("UTF-8").unwrap(), new_utf8_encoder());
  }

  #[test]
  fn reads_a_declaration_below_a_byte_order_mark_and_comment_banners() {
    let input: &[u8] =
      b"\xEF\xBB\xBF\r\n<!-- ---- banner ---- -->\r\n<!-- second -->\r\n<?xml version=\"1.0\" encoding=\"cp1251\"?>\r\n<root/>";

    assert_eq!(
      XmlEncoding::find_declared(input).unwrap(),
      Some(new_windows1251_encoder())
    );
  }

  #[test]
  fn reads_a_declaration_however_long_its_prolog() {
    let mut input: Vec<u8> = format!("<!-- {} -->\n", "=".repeat(4096)).into_bytes();

    input.extend_from_slice(b"<?xml version=\"1.0\"");
    input.extend_from_slice(&b" ".repeat(512));
    input.extend_from_slice(b"encoding=\"windows-1250\"?><root/>");

    assert_eq!(
      XmlEncoding::find_declared(&input).unwrap(),
      Some(new_windows1250_encoder())
    );
  }

  #[test]
  fn ignores_what_only_looks_like_a_declaration() {
    for input in [
      // Quoted in a comment, named by another processing instruction, or past the start of the document.
      "<!-- <?xml version=\"1.0\" encoding=\"windows-1251\"?> --><root>Привет</root>",
      "<?xml-stylesheet encoding=\"windows-1251\"?><root>Привет</root>",
      "<root><?xml version=\"1.0\" encoding=\"windows-1251\"?></root>",
      "<?xml version=\"1.0\" note=\"encoding=windows-1251\"?><root/>",
      "<!-- unterminated <?xml version=\"1.0\" encoding=\"windows-1251\"?>",
    ] {
      assert_eq!(
        XmlEncoding::find_declared(input.as_bytes()).unwrap(),
        None,
        "for {input}"
      );
    }
  }

  #[test]
  fn reads_the_encoding_whichever_pseudo_attribute_precedes_it() {
    let input: &[u8] = b"<?xml version=\"1.0\" standalone='yes' ENCODING=\"windows-1252\"?><root/>";

    assert_eq!(
      XmlEncoding::find_declared(input).unwrap(),
      Some(new_windows1252_encoder())
    );
  }
}
