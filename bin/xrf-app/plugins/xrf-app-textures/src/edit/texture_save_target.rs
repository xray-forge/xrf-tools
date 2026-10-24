use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_app_core::assets::FileStamp;
use xrf_utils::format_path;

/// One file a write addresses, and what was there when the editor read it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureSaveTarget {
  /// Absolute path of the file to replace or create.
  pub path: String,
  /// The stamp the editor read there, or `None` for a file it is creating.
  pub expected: Option<FileStamp>,
}

impl TextureSaveTarget {
  /// The file at `path`, stamped as it is now.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not Unicode, which the page could not send back as the same file, or when the
  /// file exists and its metadata cannot be read.
  pub fn of_path(path: &Path) -> TauriResult<Self> {
    Ok(Self {
      path: path
        .to_str()
        .ok_or_else(|| format!("'{}' has no Unicode path to save to", format_path(path)))?
        .to_owned(),
      expected: FileStamp::read(path)?,
    })
  }
}
