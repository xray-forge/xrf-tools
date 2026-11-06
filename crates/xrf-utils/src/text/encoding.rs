use std::borrow::Cow;
use std::io;
use std::io::{ErrorKind, Read};

use base64::engine::{GeneralPurpose, general_purpose};
use base64::{Engine, alphabet};
use encoding_rs::{Encoding, UTF_8, WINDOWS_1250, WINDOWS_1251, WINDOWS_1252};
use xrf_error::{XrfError, XrfResult};

/// A text encoding a format names, never one guessed from the bytes.
pub type XRayEncoding = &'static Encoding;

/// URL-safe Base64 without padding, the one alphabet the workspace writes.
const BASE64_ENGINE: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// The Windows-1250 code page.
#[inline]
pub fn new_windows1250_encoder() -> XRayEncoding {
  WINDOWS_1250
}

/// The Windows-1251 code page, the encoding game text is written in.
#[inline]
pub fn new_windows1251_encoder() -> XRayEncoding {
  WINDOWS_1251
}

/// The Windows-1252 code page.
#[inline]
pub fn new_windows1252_encoder() -> XRayEncoding {
  WINDOWS_1252
}

/// UTF-8, for text a format declares as Unicode.
#[inline]
pub fn new_utf8_encoder() -> XRayEncoding {
  UTF_8
}

/// Decodes `bytes` in `encoding`, honouring and removing a byte order mark, and refusing rather than substituting a
/// byte the encoding cannot read.
#[inline]
pub fn decode_bytes_to_string(bytes: &[u8], encoding: XRayEncoding) -> io::Result<String> {
  let (cow, _, had_errors) = encoding.decode(bytes);

  if had_errors {
    Err(new_decode_error(encoding, bytes.len()))
  } else {
    Ok(cow.to_string())
  }
}

/// Decodes `bytes` in `encoding` as [`decode_bytes_to_string`] does, keeping the buffer as the string when its bytes
/// already are that text: ASCII decodes to itself in every code page, so a large document is not copied to come out
/// the same.
fn decode_vec_to_string(bytes: Vec<u8>, encoding: XRayEncoding) -> io::Result<String> {
  let (cow, _, had_errors) = encoding.decode(&bytes);

  if had_errors {
    return Err(new_decode_error(encoding, bytes.len()));
  }

  // Borrowed and as long as the buffer is the buffer read as it stands; a byte order mark makes it shorter.
  let decoded: Option<String> = match cow {
    Cow::Borrowed(text) if text.len() == bytes.len() => None,
    cow => Some(cow.into_owned()),
  };

  match decoded {
    Some(decoded) => Ok(decoded),
    None => String::from_utf8(bytes).map_err(|error| io::Error::new(ErrorKind::InvalidData, error)),
  }
}

fn new_decode_error(encoding: XRayEncoding, length: usize) -> io::Error {
  io::Error::new(
    ErrorKind::InvalidData,
    format!("Failed to decode buffer data with {encoding:?} encoding, {length} bytes"),
  )
}

/// Decodes `bytes` in `encoding` as [`decode_bytes_to_string`] does, but reads a byte order mark as text.
#[inline]
pub fn decode_bytes_to_string_without_bom_handling(bytes: &[u8], encoding: XRayEncoding) -> io::Result<String> {
  let (cow, had_errors) = encoding.decode_without_bom_handling(bytes);

  if had_errors {
    Err(io::Error::new(
      ErrorKind::InvalidData,
      format!(
        "Failed to decode (no bom handling) buffer data with {:?} encoding, {} bytes",
        encoding,
        bytes.len()
      ),
    ))
  } else {
    Ok(cow.to_string())
  }
}

/// Encodes `string` in `encoding`, refusing rather than substituting a character the encoding cannot write.
#[inline]
pub fn encode_string_to_bytes(string: &str, encoding: XRayEncoding) -> io::Result<Vec<u8>> {
  let (transformed, _, had_errors) = encoding.encode(string);

  if had_errors {
    Err(new_encode_error(encoding, string.len()))
  } else {
    Ok(match transformed {
      Cow::Borrowed(value) => value.to_vec(),
      Cow::Owned(value) => value,
    })
  }
}

/// Encodes `string` in `encoding` as [`encode_string_to_bytes`] does, handing back the string's own buffer when its
/// bytes already are the encoding: ASCII encodes to itself in every code page, so a large document is not copied to
/// come out the same.
fn encode_string_into_bytes(string: String, encoding: XRayEncoding) -> io::Result<Vec<u8>> {
  let (transformed, _, had_errors) = encoding.encode(&string);

  if had_errors {
    return Err(new_encode_error(encoding, string.len()));
  }

  let encoded: Option<Vec<u8>> = match transformed {
    Cow::Borrowed(_) => None,
    Cow::Owned(value) => Some(value),
  };

  Ok(encoded.unwrap_or_else(|| string.into_bytes()))
}

fn new_encode_error(encoding: XRayEncoding, length: usize) -> io::Error {
  io::Error::new(
    ErrorKind::InvalidData,
    format!("Failed to encode string with {encoding:?} encoding, {length} characters"),
  )
}

/// Reads everything `reader` holds and decodes it as Windows-1251, the encoding game text is written in.
#[inline]
pub fn read_as_string_from_w1251_encoded<R: Read>(reader: &mut R) -> io::Result<String> {
  let mut bytes: Vec<u8> = Vec::new();

  reader.read_to_end(&mut bytes)?;

  decode_vec_to_string(bytes, WINDOWS_1251)
}

/// Decodes `bytes` as Windows-1251, the encoding game text is written in, honouring a byte order mark the way a text
/// editor saving a whole document leaves one; [`decode_w1251_field_to_string`] reads a string inside a binary format.
#[inline]
pub fn decode_w1251_bytes_to_string(bytes: &[u8]) -> io::Result<String> {
  decode_bytes_to_string(bytes, WINDOWS_1251)
}

/// Decodes a string a binary format stores, a chunk's name or a fixed-length field, as Windows-1251 throughout.
///
/// No byte order mark is looked for: such a string is never a document of its own, so `EF BB BF` at its start is
/// «п»ї» and `FF FE` is «яю», and reading either as a mark would drop or reinterpret bytes a writer then cannot put
/// back.
#[inline]
pub fn decode_w1251_field_to_string(bytes: &[u8]) -> io::Result<String> {
  decode_bytes_to_string_without_bom_handling(bytes, WINDOWS_1251)
}

/// Encodes `string` as Windows-1251, the encoding game text is written in.
#[inline]
pub fn encode_string_to_w1251_bytes(string: &str) -> io::Result<Vec<u8>> {
  encode_string_to_bytes(string, WINDOWS_1251)
}

/// Encodes `string` as Windows-1251 as [`encode_string_to_w1251_bytes`] does, reusing its buffer for ASCII text.
#[inline]
pub fn encode_string_into_w1251_bytes(string: String) -> io::Result<Vec<u8>> {
  encode_string_into_bytes(string, WINDOWS_1251)
}

/// Encodes text as URL-safe Base64 without padding.
#[inline]
pub fn encode_string_to_base64(string: &str) -> String {
  BASE64_ENGINE.encode(string)
}

/// Encodes bytes as URL-safe Base64 without padding.
#[inline]
pub fn encode_bytes_to_base64(bytes: &[u8]) -> String {
  BASE64_ENGINE.encode(bytes)
}

/// Decodes URL-safe Base64 without padding into bytes.
#[inline]
pub fn decode_bytes_from_base64(string: &str) -> XrfResult<Vec<u8>> {
  BASE64_ENGINE
    .decode(string)
    .map_err(|error| XrfError::new_parsing_error(format!("Failed to decode bytes value from base 64: {}", error)))
}

/// Decodes URL-safe Base64 without padding into text, refusing bytes that are not UTF-8.
#[inline]
pub fn decode_string_from_base64(string: &str) -> XrfResult<String> {
  let bytes: Vec<u8> = BASE64_ENGINE
    .decode(string)
    .map_err(|error| XrfError::new_parsing_error(format!("Failed to decode string value from base 64: {error}")))?;

  String::from_utf8(bytes)
    .map_err(|error| XrfError::new_parsing_error(format!("Decoded base 64 string value is not valid UTF-8: {error}")))
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;

  use super::{
    decode_string_from_base64, decode_w1251_bytes_to_string, decode_w1251_field_to_string,
    encode_string_into_w1251_bytes, encode_string_to_base64, encode_string_to_w1251_bytes,
    read_as_string_from_w1251_encoded,
  };

  #[test]
  fn encoding_into_bytes_keeps_ascii_text_as_it_is_and_encodes_the_rest() {
    let ascii: String = String::from("[section]\r\nkey = value\r\n");
    let address: *const u8 = ascii.as_ptr();
    let encoded: Vec<u8> = encode_string_into_w1251_bytes(ascii.clone()).expect("ASCII encodes");

    assert_eq!(encoded, ascii.as_bytes());
    assert_ne!(encoded.as_ptr(), address, "the clone handed over, not the original");

    let handed: Vec<u8> = encode_string_into_w1251_bytes(ascii).expect("ASCII encodes");

    assert_eq!(handed.as_ptr(), address, "ASCII text to come back in its own buffer");

    for text in ["мова = Жовтий\r\n", "a\u{0451}b"] {
      assert_eq!(
        encode_string_into_w1251_bytes(String::from(text)).expect("Windows-1251 text encodes"),
        encode_string_to_w1251_bytes(text).expect("Windows-1251 text encodes")
      );
    }

    assert_eq!(
      encode_string_into_w1251_bytes(String::from("åäö"))
        .unwrap_err()
        .to_string(),
      encode_string_to_w1251_bytes("åäö").unwrap_err().to_string()
    );
  }

  #[test]
  fn reading_decodes_as_the_slice_reader_does() {
    for bytes in [
      &b"[section]\r\nkey = value\r\n"[..],
      &[0xec, 0xee, 0xe2, 0xe0, b' ', b'=', b' ', 0xc6][..],
      &[0xef, 0xbb, 0xbf, b'a', b'b'][..],
      &[0xef, 0xbb, 0xbf, 0xd0, 0x96][..],
      &[0xff, 0xfe, b'a', 0x00][..],
      &[][..],
    ] {
      assert_eq!(
        read_as_string_from_w1251_encoded(&mut &bytes[..]).expect("decodes"),
        decode_w1251_bytes_to_string(bytes).expect("decodes"),
        "{bytes:?}"
      );
    }

    // A byte order mark announcing UTF-8 ahead of bytes that are not is refused by both.
    assert!(read_as_string_from_w1251_encoded(&mut &[0xef, 0xbb, 0xbf, 0xff][..]).is_err());
    assert!(decode_w1251_bytes_to_string(&[0xef, 0xbb, 0xbf, 0xff]).is_err());
  }

  #[test]
  fn a_field_opening_on_byte_order_mark_bytes_reads_as_its_letters() {
    // `EF BB BF` and `FF FE` are «п»ї» and «яю» in Windows-1251, and a field holds nothing else.
    for bytes in [
      &[0xef, 0xbb, 0xbf, b'a'][..],
      &[0xff, 0xfe, b'a'][..],
      &[0xfe, 0xff, b'a', b'b'][..],
    ] {
      let value: String = decode_w1251_field_to_string(bytes).expect("every byte is Windows-1251");

      assert_eq!(encode_string_to_w1251_bytes(&value).expect("encodes"), bytes, "{value}");
    }

    assert_eq!(
      decode_w1251_field_to_string(&[0xef, 0xbb, 0xbf, b'a']).expect("decodes"),
      "п»їa"
    );
  }

  #[test]
  fn a_document_honours_a_byte_order_mark() {
    // A text editor saving a whole LTX may leave one, and the document reader keeps reading such a file as before.
    assert_eq!(
      decode_w1251_bytes_to_string(&[0xef, 0xbb, 0xbf, b'a']).expect("decodes"),
      "a"
    );
  }

  #[test]
  fn decode_string_from_base64_rejects_invalid_utf8() {
    assert!(matches!(
      decode_string_from_base64("_w"),
      Err(XrfError::Parsing { message }) if message.contains("not valid UTF-8")
    ));
  }

  #[test]
  fn decode_string_from_base64_round_trips_valid_utf8() {
    let original: &str = "\u{041f}\u{0440}\u{0438}\u{0432}\u{0456}\u{0442}";

    assert!(matches!(
      decode_string_from_base64(&encode_string_to_base64(original)),
      Ok(value) if value == original
    ));
  }
}
