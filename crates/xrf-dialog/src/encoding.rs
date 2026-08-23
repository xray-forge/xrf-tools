use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, decode_bytes_to_string_without_bom_handling, new_utf8_encoder, new_windows1251_encoder};
use xrf_xml::declared_xml_encoding;

/// A dialog file's bytes, decoded, with what is needed to write them back.
pub(crate) struct DecodedDialogSource {
  pub byte_order_mark: Vec<u8>,
  pub encoding: XRayEncoding,
  pub text: String,
}

/// Decide which encoding a dialog file is written in.
///
/// Resolved once and kept, so reading and writing cannot disagree: deciding it separately in each
/// direction would decode a file one way and re-encode it another, corrupting bytes no edit touched.
///
/// The declaration wins where there is one. Shipped dialog XML always declares `windows-1251`, which
/// is also the fallback, because that is what the engine assumes for gameplay configs.
///
/// # Errors
///
/// Returns an encoding error when the declaration names a code page there is no encoder for.
pub(crate) fn resolve_encoding(data: &[u8]) -> XrfResult<XRayEncoding> {
  Ok(declared_xml_encoding(data)?.unwrap_or_else(new_windows1251_encoder))
}

/// Read a dialog file into text.
///
/// # Errors
///
/// Returns an IO error when the file cannot be read, and an encoding error when the bytes are UTF-16
/// or do not decode as the encoding they claim.
pub(crate) fn read_decoded(path: &Path) -> XrfResult<DecodedDialogSource> {
  decode(&fs::read(path)?)
}

/// Decode dialog bytes that are already in hand, such as an archived entry.
///
/// # Errors
///
/// Returns an encoding error when the bytes are UTF-16 or do not decode as the encoding they claim.
pub(crate) fn decode(data: &[u8]) -> XrfResult<DecodedDialogSource> {
  let (mark, body) = split_byte_order_mark(data)?;

  // A byte order mark decides the encoding and outranks the declaration, because shipped files
  // contradict themselves: a UTF-8 marked file declaring windows-1251 exists in reference gamedata.
  // The mark is also kept out of the decoded text and put back verbatim, since the usual decode
  // strips it and re-encoding would then drop it from a file that had one.
  let encoding: XRayEncoding = if mark.is_empty() {
    resolve_encoding(body)?
  } else {
    new_utf8_encoder()
  };

  Ok(DecodedDialogSource {
    byte_order_mark: mark.to_vec(),
    encoding,
    text: decode_bytes_to_string_without_bom_handling(body, encoding)?,
  })
}

/// Split a leading byte order mark off the content, so it survives a rewrite untouched.
///
/// # Errors
///
/// Returns an encoding error for UTF-16 content, which no gameplay config encoder can hold and which
/// decoding as a code page would silently mangle.
fn split_byte_order_mark(data: &[u8]) -> XrfResult<(&[u8], &[u8])> {
  if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
    return Ok(data.split_at(3));
  }

  if data.starts_with(&[0xFF, 0xFE]) || data.starts_with(&[0xFE, 0xFF]) {
    return Err(XrfError::new_encoding_error(
      "Dialog XML is UTF-16, which gameplay configs cannot be written as",
    ));
  }

  Ok((&[], data))
}
