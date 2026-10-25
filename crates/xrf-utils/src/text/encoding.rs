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
    Err(io::Error::new(
      ErrorKind::InvalidData,
      format!(
        "Failed to decode buffer data with {:?} encoding, {} bytes",
        encoding,
        bytes.len()
      ),
    ))
  } else {
    Ok(cow.to_string())
  }
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
    Err(io::Error::new(
      ErrorKind::InvalidData,
      format!(
        "Failed to encode string with {:?} encoding, {} characters",
        encoding,
        string.len()
      ),
    ))
  } else {
    Ok(match transformed {
      Cow::Borrowed(value) => value.to_vec(),
      Cow::Owned(value) => value,
    })
  }
}

/// Reads everything `reader` holds and decodes it as Windows-1251, the encoding game text is written in.
#[inline]
pub fn read_as_string_from_w1251_encoded<R: Read>(reader: &mut R) -> io::Result<String> {
  let mut bytes: Vec<u8> = Vec::new();

  reader.read_to_end(&mut bytes)?;

  decode_w1251_bytes_to_string(&bytes)
}

/// Decodes `bytes` as Windows-1251, the encoding game text is written in.
#[inline]
pub fn decode_w1251_bytes_to_string(bytes: &[u8]) -> io::Result<String> {
  decode_bytes_to_string(bytes, WINDOWS_1251)
}

/// Encodes `string` as Windows-1251, the encoding game text is written in.
#[inline]
pub fn encode_string_to_w1251_bytes(string: &str) -> io::Result<Vec<u8>> {
  encode_string_to_bytes(string, WINDOWS_1251)
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

  use super::{decode_string_from_base64, encode_string_to_base64};

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
