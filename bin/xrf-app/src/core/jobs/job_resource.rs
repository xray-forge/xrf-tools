use std::path::{Path, PathBuf};

use crate::core::jobs::lease_path::resolve_lease_path;
use crate::core::types::TauriResult;

/// A write destination, independent of the command that writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobResource {
  File(PathBuf),
  Tree(PathBuf),
}

impl JobResource {
  pub fn file(path: impl Into<PathBuf>) -> Self {
    Self::File(path.into())
  }
  pub fn tree(path: impl Into<PathBuf>) -> Self {
    Self::Tree(path.into())
  }

  /// Resolve before registration takes the lock; filesystem failures refuse the start.
  pub(super) fn resolve(&self) -> TauriResult<Self> {
    let path: PathBuf = resolve_lease_path(self.path())?;
    Ok(match self {
      Self::File(_) => Self::File(path),
      Self::Tree(_) => Self::Tree(path),
    })
  }

  /// Compare resolved paths on component boundaries, including containment in either direction.
  pub(super) fn overlaps(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::File(left), Self::File(right)) => left == right,
      (Self::Tree(tree), Self::File(file)) | (Self::File(file), Self::Tree(tree)) => file.starts_with(tree),
      (Self::Tree(left), Self::Tree(right)) => left.starts_with(right) || right.starts_with(left),
    }
  }

  pub(super) fn describe(&self) -> String {
    let kind: &str = match self {
      Self::File(_) => "file",
      Self::Tree(_) => "tree",
    };
    format!("{kind}:{}", self.path().display())
  }

  fn path(&self) -> &Path {
    match self {
      Self::File(path) | Self::Tree(path) => path,
    }
  }
}
