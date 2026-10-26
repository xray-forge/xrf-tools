use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, decode_bytes_to_string_without_bom_handling, encode_string_to_bytes, new_utf8_encoder};

use crate::xml::translation_identity::TranslationIdentity;

/// A string table's bytes, decoded, with what is needed to write them back.
pub(crate) struct DecodedTranslation {
  pub byte_order_mark: Vec<u8>,
  pub encoding: XRayEncoding,
  pub text: String,
}

impl DecodedTranslation {
  /// Decode a string table already in hand, such as an entry read out of an archive.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when the bytes are UTF-16 or do not decode as the encoding they resolve to.
  pub(crate) fn decode(identity: TranslationIdentity, data: &[u8]) -> XrfResult<Self> {
    let (mark, body) = Self::split_byte_order_mark(identity.file_name, data)?;

    // A byte order mark decides the encoding and outranks everything else, because it is the one statement about the
    // bytes no tool writes by accident. It is also kept out of the decoded text and put back verbatim, because the
    // usual decode strips it and re-encoding would then drop it from a file that had one.
    let encoding: XRayEncoding = if mark.is_empty() {
      identity.resolve_encoding(body)?
    } else {
      new_utf8_encoder()
    };

    Ok(Self {
      byte_order_mark: mark.to_vec(),
      encoding,
      text: decode_bytes_to_string_without_bom_handling(body, encoding)?,
    })
  }

  /// Read a string table off disk and decode it.
  ///
  /// For a caller that holds one file and no mounted roots; everything reading a tree goes through the VFS and calls
  /// [`Self::decode`] with the bytes it already has.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the file cannot be read, and an encoding error when its bytes do not decode.
  pub(crate) fn read(path: &Path) -> XrfResult<Self> {
    Self::decode(TranslationIdentity::from_path(path), &fs::read(path)?)
  }

  /// The bytes `text` is written back as: this table's byte order mark, then `text` in this table's encoding.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when `text` holds a character the encoding cannot.
  pub(crate) fn encode(&self, text: &str) -> XrfResult<Vec<u8>> {
    let mut written: Vec<u8> = self.byte_order_mark.clone();

    written.extend(encode_string_to_bytes(text, self.encoding)?);

    Ok(written)
  }

  /// Split a leading byte order mark off the content, so it survives a rewrite untouched.
  ///
  /// # Errors
  ///
  /// Returns an encoding error for UTF-16 content, which none of the string table encoders can hold and which decoding
  /// as a code page would silently mangle.
  fn split_byte_order_mark<'a>(subject: &str, data: &'a [u8]) -> XrfResult<(&'a [u8], &'a [u8])> {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
      return Ok(data.split_at(3));
    }

    if data.starts_with(&[0xFF, 0xFE]) || data.starts_with(&[0xFE, 0xFF]) {
      return Err(XrfError::new_encoding_error(format!(
        "Translation '{subject}' is UTF-16, which string tables cannot be written as"
      )));
    }

    Ok((&[], data))
  }
}
