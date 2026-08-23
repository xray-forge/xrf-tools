use std::borrow::Cow;
use std::io;
use std::io::{ErrorKind, Read};

use base64::engine::{GeneralPurpose, general_purpose};
use base64::{Engine, alphabet};
use encoding_rs::{Encoding, UTF_8};
use encoding_rs::{WINDOWS_1250, WINDOWS_1251, WINDOWS_1252};
use xrf_error::{XrfError, XrfResult};

pub type XRayEncoding = &'static Encoding;

pub const CUSTOM_B64_ENGINE: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Return encoding factory for windows1250.
#[inline]
pub fn new_windows1250_encoder() -> XRayEncoding {
  WINDOWS_1250
}

/// Return encoding factory for windows1251.
#[inline]
pub fn new_windows1251_encoder() -> XRayEncoding {
  WINDOWS_1251
}

/// Return encoding factory for windows1252.
#[inline]
pub fn new_windows1252_encoder() -> XRayEncoding {
  WINDOWS_1252
}

/// Return encoding factory for UTF-8.
#[inline]
pub fn new_utf8_encoder() -> XRayEncoding {
  UTF_8
}

/// Try encoding provided u8 raw bytes as string value.
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

/// Try encoding provided u8 raw bytes as string value.
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

/// Try encoding provided string value as bytes.
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

/// Read whole encoded reader content to a string.
#[inline]
pub fn read_as_string_from_encoded<R: Read>(reader: &mut R, encoding: XRayEncoding) -> io::Result<String> {
  let mut raw_data: Vec<u8> = Vec::new();

  reader.read_to_end(&mut raw_data)?;

  decode_bytes_to_string(&raw_data, encoding)
}

/// Try encoding provided u8 raw bytes as window 1251 string and convert into regular rust string.
#[inline]
pub fn read_as_string_from_w1251_encoded<R: Read>(reader: &mut R) -> io::Result<String> {
  read_as_string_from_encoded(reader, WINDOWS_1251)
}

/// Try encoding provided u8 raw bytes as window 1251 string and convert into regular rust string.
#[inline]
pub fn encode_w1251_bytes_to_string(bytes: &[u8]) -> io::Result<String> {
  decode_bytes_to_string(bytes, WINDOWS_1251)
}

/// Try encoding provided string as windows1251 bytes.
#[inline]
pub fn encode_string_to_w1251_bytes(string: &str) -> io::Result<Vec<u8>> {
  encode_string_to_bytes(string, WINDOWS_1251)
}

/// Encode str as b64 value.
#[inline]
pub fn encode_string_to_base64(string: &str) -> String {
  CUSTOM_B64_ENGINE.encode(string)
}

/// Encode bytes as b64 value.
#[inline]
pub fn encode_bytes_to_base64(bytes: &[u8]) -> String {
  CUSTOM_B64_ENGINE.encode(bytes)
}

/// Decode b64 as bytes.
#[inline]
pub fn decode_bytes_from_base64(string: &str) -> XrfResult<Vec<u8>> {
  CUSTOM_B64_ENGINE
    .decode(string)
    .map_err(|error| XrfError::new_parsing_error(format!("Failed to decode bytes value from base 64: {}", error)))
}

/// Decode b64 as string.
#[inline]
pub fn decode_string_from_base64(string: &str) -> XrfResult<String> {
  Ok(match CUSTOM_B64_ENGINE.decode(string) {
    Ok(value) => String::from_utf8_lossy(&value).into_owned(),
    Err(error) => {
      return Err(XrfError::new_parsing_error(format!(
        "Failed to decode string value from base 64: {}",
        error
      )));
    }
  })
}
