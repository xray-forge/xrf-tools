use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, decode_bytes_to_string_without_bom_handling};
use xrf_xml::XmlEngineSource;

/// A dialog file's bytes, decoded, with what is needed to write them back.
pub(crate) struct DialogSource {
  pub byte_order_mark: Vec<u8>,
  pub encoding: XRayEncoding,
  pub text: String,
}

impl DialogSource {
  /// Read a dialog file into text.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the file cannot be read, and an encoding error when the bytes are UTF-16
  /// or do not decode as the encoding they claim.
  pub(crate) fn read(path: &Path) -> XrfResult<Self> {
    Self::decode(&fs::read(path)?)
  }

  /// Decode dialog bytes that are already in hand, such as an archived entry.
  ///
  /// The encoding is the one the engine reads a config in ([`XmlEngineSource::find_encoding`]), resolved once and
  /// kept, so reading and writing cannot disagree: deciding it separately in each direction would decode a file one
  /// way and re-encode it another, corrupting bytes no edit touched. A byte order mark is kept out of the text and put
  /// back verbatim, since re-encoding would otherwise drop it from a file that had one.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when the bytes are UTF-16 or do not decode as the encoding they claim.
  pub(crate) fn decode(data: &[u8]) -> XrfResult<Self> {
    let encoding: XRayEncoding = XmlEngineSource::find_encoding(data)?;
    let (mark, body) = XmlEngineSource::split_byte_order_mark(data);

    Ok(Self {
      byte_order_mark: mark.to_vec(),
      encoding,
      text: decode_bytes_to_string_without_bom_handling(body, encoding)
        .map_err(|error| XrfError::new_encoding_error(error.to_string()))?,
    })
  }
}
