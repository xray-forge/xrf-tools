use std::cmp::Ordering;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{decode_w1251_field_to_string, encode_string_into_w1251_bytes, encode_string_to_w1251_bytes};

/// How a file's text is stored as bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IniEncoding {
  Utf8,
  /// The engine's and the SDK's text encoding.
  Windows1251,
}

impl IniEncoding {
  /// # Errors
  ///
  /// Returns an encoding error when the bytes are not valid in this encoding.
  pub fn decode(self, bytes: &[u8]) -> XrfResult<String> {
    match self {
      Self::Utf8 => String::from_utf8(bytes.to_vec())
        .map_err(|error| XrfError::new_encoding_error(format!("INI text is not UTF-8: {error}"))),
      Self::Windows1251 => Ok(decode_w1251_field_to_string(bytes)?),
    }
  }

  /// # Errors
  ///
  /// Returns an encoding error when the text holds a character this encoding has no byte for.
  pub fn encode(self, text: String) -> XrfResult<Vec<u8>> {
    match self {
      Self::Utf8 => Ok(text.into_bytes()),
      Self::Windows1251 => Ok(encode_string_into_w1251_bytes(text)?),
    }
  }

  /// Orders two names as `strcmp` orders their bytes in this encoding.
  pub fn compare(self, left: &str, right: &str) -> Ordering {
    match self {
      Self::Utf8 => left.cmp(right),
      Self::Windows1251 => match (encode_string_to_w1251_bytes(left), encode_string_to_w1251_bytes(right)) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => left.cmp(right),
      },
    }
  }
}
