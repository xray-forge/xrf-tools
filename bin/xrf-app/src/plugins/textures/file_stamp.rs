//! What a file looked like when the editor read it, and what it means when it no longer looks like that.

use std::fs::Metadata;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use xrf_utils::format_path;

use crate::core::types::TauriResult;

/// What the editor read at a path, so a later write cannot overwrite a change it never saw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureFileStamp {
  pub size: u64,
  /// Milliseconds since the Unix epoch, as the platform reports the file's modification time.
  pub modified_ms: u64,
}

impl TextureFileStamp {
  /// The stamp a path currently carries, or `None` when nothing is there.
  ///
  /// # Errors
  ///
  /// Returns an error when the path exists and cannot be read, which is not the same as it being absent: a file behind
  /// a permission wall is a write that must refuse rather than one that may create.
  pub fn read(path: &Path) -> TauriResult<Option<Self>> {
    match std::fs::metadata(path) {
      Ok(metadata) => Ok(Some(Self::of(&metadata))),
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
      Err(error) => Err(format!("Cannot read '{}': {error}", format_path(path))),
    }
  }

  /// Refuse a path whose file is no longer the one `expected` describes.
  ///
  /// # Errors
  ///
  /// Returns an error naming the file and naming reloading, because reloading is the only correct move.
  pub fn check_unchanged(path: &Path, expected: Option<Self>) -> TauriResult<()> {
    if Self::read(path)? == expected {
      return Ok(());
    }

    Err(format!("'{}' changed on disk, reload before saving", format_path(path)))
  }

  fn of(metadata: &Metadata) -> Self {
    Self {
      size: metadata.len(),
      // A filesystem that reports no modification time leaves the size as the whole guard rather than failing the
      // write. The case is a network mount, and a texture that changed size is the change worth catching either way.
      modified_ms: metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX)),
    }
  }
}
