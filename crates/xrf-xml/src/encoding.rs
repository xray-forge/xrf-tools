use xrf_error::{XrfError, XrfResult};
use xrf_utils::{
  XRayEncoding, decode_bytes_to_string, new_utf8_encoder, new_windows1250_encoder, new_windows1251_encoder,
  new_windows1252_encoder,
};

/// How far into a document the declaration is looked for.
const DECLARATION_SCAN_LIMIT: usize = 256;

/// Return the encoding declared by an XML prolog, if present.
///
/// # Errors
///
/// Returns an encoding error when the declaration names a code page there is no encoder for.
pub fn declared_xml_encoding(input: &[u8]) -> XrfResult<Option<XRayEncoding>> {
  let Some(label) = declared_encoding_label(input) else {
    return Ok(None);
  };

  Ok(Some(encoding_from_label(&label)?))
}

/// Resolve an encoding name to the encoder that reads and writes it.
///
/// # Errors
///
/// Returns an encoding error for a name none of the X-Ray encoders cover.
pub fn encoding_from_label(label: &str) -> XrfResult<XRayEncoding> {
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
pub(crate) fn decode_xml_bytes(input: &[u8]) -> XrfResult<String> {
  // Mapped rather than converted: bytes that do not decode are the document's own defect, and `From<io::Error>` would
  // report them as an IO failure, which a caller reads as the file never having been read.
  decode_bytes_to_string(input, declared_xml_encoding(input)?.unwrap_or_else(new_utf8_encoder))
    .map_err(|error| XrfError::new_encoding_error(error.to_string()))
}

/// Pull the raw encoding name out of a prolog, without judging whether it is supported.
///
/// Scanned by hand rather than parsed, because the declaration has to be read before the document can
/// be decoded, and decoding is what the declaration decides.
fn declared_encoding_label(input: &[u8]) -> Option<String> {
  let prefix_length: usize = input.len().min(DECLARATION_SCAN_LIMIT);
  let prefix: String = String::from_utf8_lossy(&input[..prefix_length]).into_owned();
  let lowercase: String = prefix.to_ascii_lowercase();
  let declaration_start: usize = lowercase.find("<?xml")?;
  let declaration_end: usize = lowercase[declaration_start..].find("?>")? + declaration_start;
  let declaration: &str = &prefix[declaration_start..declaration_end];
  let declaration_lowercase: String = declaration.to_ascii_lowercase();
  let encoding_start: usize = declaration_lowercase.find("encoding")? + "encoding".len();
  let after_encoding: &str = declaration[encoding_start..].trim_start();
  let after_equals: &str = after_encoding.strip_prefix('=')?.trim_start();
  let quote: char = after_equals.chars().next()?;

  if quote != '\'' && quote != '"' {
    return None;
  }

  let value: &str = &after_equals[quote.len_utf8()..];
  let end: usize = value.find(quote)?;

  Some(value[..end].to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_a_declared_encoding() {
    let input: &[u8] = b"<?xml version=\"1.0\" encoding=\"windows-1251\"?><root/>";

    assert_eq!(declared_xml_encoding(input).unwrap(), Some(new_windows1251_encoder()));
  }

  #[test]
  fn accepts_single_quotes_and_odd_spacing() {
    let input: &[u8] = b"<?xml version='1.0' encoding = 'cp1250' ?><root/>";

    assert_eq!(declared_xml_encoding(input).unwrap(), Some(new_windows1250_encoder()));
  }

  #[test]
  fn reports_nothing_when_there_is_no_declaration() {
    assert_eq!(declared_xml_encoding(b"<root/>").unwrap(), None);
  }

  #[test]
  fn refuses_an_encoding_with_no_encoder() {
    let error = declared_xml_encoding(b"<?xml version=\"1.0\" encoding=\"shift_jis\"?><root/>").unwrap_err();

    assert!(error.to_string().contains("Unsupported XML encoding 'shift_jis'"));
  }

  #[test]
  fn normalizes_punctuation_and_case_in_a_label() {
    assert_eq!(encoding_from_label("WINDOWS-1252").unwrap(), new_windows1252_encoder());
    assert_eq!(encoding_from_label("cp_1251").unwrap(), new_windows1251_encoder());
    assert_eq!(encoding_from_label("UTF-8").unwrap(), new_utf8_encoder());
  }

  #[test]
  fn ignores_a_declaration_beyond_the_scan_limit() {
    let mut input: Vec<u8> = vec![b' '; DECLARATION_SCAN_LIMIT];

    input.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"windows-1251\"?><root/>");

    // Bounded on purpose: the declaration is a prolog, and scanning a whole file for one would let a
    // stray string deep inside the document decide how the document is read.
    assert_eq!(declared_xml_encoding(&input).unwrap(), None);
  }
}
