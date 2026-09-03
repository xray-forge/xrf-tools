use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// How a packing configuration file is serialized.
///
/// Two spellings of one payload: the xrCompress dialect every existing configuration is written in, and JSON, which
/// automation can produce without knowing that dialect. Neither is the payload itself; this only says which encoding
/// a given file holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchivePackConfigFormat {
  /// The `ProcessLTX` dialect, with `[include_folders]` and a verbatim `[header]`.
  Ltx,
  /// The shape [`crate::ArchivePackConfigJson`] serializes to.
  Json,
}

impl ArchivePackConfigFormat {
  pub const EXTENSION_LTX: &'static str = "ltx";
  pub const EXTENSION_JSON: &'static str = "json";

  /// Select the codec from a path's extension.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the path carries no extension or one this does not serialize, naming both
  /// supported formats.
  pub fn from_path<P: AsRef<Path>>(path: P) -> XrfResult<Self> {
    let path: &Path = path.as_ref();

    match path
      .extension()
      .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
      .as_deref()
    {
      Some(Self::EXTENSION_LTX) => Ok(Self::Ltx),
      Some(Self::EXTENSION_JSON) => Ok(Self::Json),
      Some(extension) => Err(XrfError::new_invalid_error(format!(
        "Cannot read '{}' as a packing configuration: '.{extension}' is not a supported format. Name it '.{}' or \
         '.{}'.",
        format_path(path),
        Self::EXTENSION_LTX,
        Self::EXTENSION_JSON
      ))),
      None => Err(XrfError::new_invalid_error(format!(
        "Cannot read '{}' as a packing configuration: it has no extension, and the format is taken from one. Name \
         it '.{}' or '.{}'.",
        format_path(path),
        Self::EXTENSION_LTX,
        Self::EXTENSION_JSON
      ))),
    }
  }
}
