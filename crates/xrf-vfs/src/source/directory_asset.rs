use std::path::{Path, PathBuf};

/// One file discovered beneath an indexed physical root.
#[derive(Clone, Debug)]
pub struct DirectoryAsset {
  relative_path: PathBuf,
}

impl DirectoryAsset {
  pub(crate) fn new(relative_path: PathBuf) -> Self {
    Self { relative_path }
  }

  pub fn relative_path(&self) -> &Path {
    &self.relative_path
  }
}
