use std::path::Path;
use std::{fs, io};

use serde::Serialize;

use crate::paths::description::PathKind;

/// What a path is right now, for a form field that has to say what it points at before a command runs.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathDescription {
  pub kind: PathKind,
  /// Entries directly inside a directory.
  pub entry_count: Option<u64>,
}

impl PathDescription {
  /// Reads what a path currently is.
  ///
  /// # Errors
  ///
  /// Returns an error when the path exists but its metadata cannot be read.
  pub fn read(target: &Path) -> io::Result<Self> {
    let metadata: fs::Metadata = match fs::metadata(target) {
      Ok(metadata) => metadata,
      Err(error) if error.kind() == io::ErrorKind::NotFound => {
        return Ok(Self {
          kind: PathKind::Missing,
          entry_count: None,
        });
      }
      Err(error) => return Err(error),
    };

    if !metadata.is_dir() {
      return Ok(Self {
        kind: PathKind::File,
        entry_count: None,
      });
    }

    Ok(Self {
      kind: PathKind::Directory,
      entry_count: Self::count_entries(target),
    })
  }

  /// Counts what a directory lists, or answers `None` where it cannot be listed at all.
  fn count_entries(directory: &Path) -> Option<u64> {
    Some(fs::read_dir(directory).ok()?.flatten().count() as u64)
  }
}
