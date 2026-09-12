use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_utils::encode_w1251_bytes_to_string;

/// One text file read for display: its name, decoded content, and unpacked size.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReadResult {
  /// Name the content was read under.
  pub name: String,
  /// Text decoded from Windows-1251, like every engine text format.
  pub content: String,
  /// Bytes once unpacked, before decoding.
  pub size: u32,
}

impl ArchiveReadResult {
  /// Decodes what a viewer will show from the bytes a source answered with.
  ///
  /// The one place engine text becomes a string for display, because both readers behind it — a volume set's name
  /// table and a mounted world's winner — meet the same encoding and used to each decide it for themselves.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when the bytes are not decodable Windows-1251.
  pub fn decode(name: &str, bytes: &[u8], size: u32) -> XrfResult<Self> {
    Ok(Self {
      name: name.into(),
      content: encode_w1251_bytes_to_string(bytes)?,
      size,
    })
  }
}
