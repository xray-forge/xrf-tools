use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use crate::core::types::TauriResult;

/// Resolve existing ancestors before appending missing output components.
///
/// Missing destinations keep their identity after creation. This is a comparison key only; authored paths still
/// address I/O. External changes to symlinks or mounts during a job are outside the registry's exclusion contract.
pub fn resolve_lease_path(path: &Path) -> TauriResult<PathBuf> {
  let resolved: PathBuf = resolve_host_path(path)
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
        resolved = resolve_existing_prefix(&resolved)?;
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
    Ok(_) => return Err(io::Error::new(ErrorKind::NotFound, "destination cannot be resolved")),
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
