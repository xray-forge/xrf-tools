use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use crate::TauriResult;

/// A write destination, independent of the command that writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobResource {
  File(PathBuf),
  Tree(PathBuf),
}

impl JobResource {
  /// One file the job writes, which no other job may write meanwhile.
  pub fn file(path: impl Into<PathBuf>) -> Self {
    Self::File(path.into())
  }

  /// A directory the job writes into, which no other job may write in or around meanwhile.
  pub fn tree(path: impl Into<PathBuf>) -> Self {
    Self::Tree(path.into())
  }

  /// The key `path` is compared by: its existing ancestors resolved, its missing components appended, so a destination
  /// created later keeps its identity. Authored paths still address I/O; changes to links or mounts during a job are
  /// outside the registry's exclusion.
  ///
  /// # Errors
  ///
  /// Returns an error where the path cannot be resolved, or cannot be compared on a platform comparing without case.
  pub fn resolve_path(path: &Path) -> TauriResult<PathBuf> {
    let resolved: PathBuf = Self::resolve_host_path(path)
      .map_err(|error| format!("Cannot resolve write destination '{}': {error}", path.display()))?;

    // Windows paths compare without case; Unix paths retain both case and non-Unicode bytes.
    #[cfg(windows)]
    let resolved: PathBuf = PathBuf::from(
      resolved
        .to_str()
        .ok_or_else(|| format!("Cannot compare non-Unicode write destination '{}'", path.display()))?
        .to_lowercase(),
    );

    Ok(resolved)
  }

  /// Resolve before registration takes the lock; filesystem failures refuse the start.
  pub(super) fn resolve(&self) -> TauriResult<Self> {
    let path: PathBuf = Self::resolve_path(self.path())?;

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

  fn resolve_host_path(path: &Path) -> io::Result<PathBuf> {
    let absolute: PathBuf = std::path::absolute(path)?;
    let mut resolved: PathBuf = PathBuf::new();

    for component in absolute.components() {
      match component {
        Component::CurDir => {}
        Component::ParentDir => {
          resolved.pop();
        }
        Component::Normal(_) => {
          resolved.push(component);
          resolved = Self::resolve_existing_prefix(&resolved)?;
        }
        Component::Prefix(_) => resolved.push(component),
        Component::RootDir => {
          resolved.push(component);
          // Canonicalize roots too: Windows returns verbatim paths for children, and their roots must match.
          resolved = resolved.canonicalize()?;
        }
      }
    }

    Ok(resolved)
  }

  /// Leave only an absent path unresolved. A dangling link, unreadable path or file used as a directory is refused.
  fn resolve_existing_prefix(path: &Path) -> io::Result<PathBuf> {
    match path.canonicalize() {
      Ok(canonical) => return Ok(canonical),
      Err(error) if error.kind() == ErrorKind::NotFound => {}
      Err(error) => return Err(error),
    }

    match path.symlink_metadata() {
      Ok(metadata) if metadata.file_type().is_symlink() => {
        return Err(io::Error::new(ErrorKind::NotFound, "destination cannot be resolved"));
      }
      // Created after the failed resolution, by another job or another process: it resolves now.
      Ok(_) => return path.canonicalize(),
      Err(error) if error.kind() == ErrorKind::NotFound => {}
      Err(error) => return Err(error),
    }

    if let Some(parent) = path.parent() {
      match parent.metadata() {
        Ok(metadata) if !metadata.is_dir() => {
          return Err(io::Error::new(
            ErrorKind::NotADirectory,
            "an ancestor is not a directory",
          ));
        }
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
        _ => {}
      }
    }

    Ok(path.to_path_buf())
  }
}
