use std::path::{Path, PathBuf};

use crate::dialog_file::DialogFile;

/// One file the project holds, parsed, with where the engine found it.
#[derive(Debug)]
pub struct DialogProjectFile {
  pub(super) logical_path: String,
  pub(super) physical_path: Option<PathBuf>,
  pub(super) file: DialogFile,
}

impl DialogProjectFile {
  /// The engine identity, which is how the project keys it.
  pub fn get_logical_path(&self) -> &str {
    &self.logical_path
  }

  /// The host path, when the winning mount is a loose directory.
  pub fn get_physical_path(&self) -> Option<&Path> {
    self.physical_path.as_deref()
  }

  /// Whether an edit could write this file back.
  pub fn is_editable(&self) -> bool {
    self.physical_path.is_some()
  }

  pub fn get_file(&self) -> &DialogFile {
    &self.file
  }
}
