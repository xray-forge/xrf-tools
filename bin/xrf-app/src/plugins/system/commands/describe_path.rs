use std::path::Path;
use std::{fs, io};

use serde::Serialize;

use crate::core::types::TauriResult;

/// What a path is right now, for a form field that has to say what it points at before a command runs.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathDescription {
  pub kind: PathKind,
  /// Entries directly inside a directory.
  pub entry_count: Option<u64>,
}

/// What a path turned out to be.
///
/// Carried instead of a pair of booleans so the states cannot disagree. Failing to look is not a variant here: it
/// reaches the caller as an error, which is what lets a refused check read as unknown rather than as absent.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PathKind {
  Missing,
  File,
  Directory,
}

/// Describe what a path currently holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_path"))]
#[tauri::command(rename = "describe_path")]
pub async fn system_describe_path(path: &str) -> TauriResult<PathDescription> {
  describe(Path::new(path)).map_err(|error| format!("Could not read {path}: {error}"))
}

/// Reads what a path is, separated from the command so it can be tested without a runtime.
fn describe(target: &Path) -> io::Result<PathDescription> {
  let metadata: fs::Metadata = match fs::metadata(target) {
    Ok(metadata) => metadata,
    Err(error) if error.kind() == io::ErrorKind::NotFound => {
      return Ok(PathDescription {
        kind: PathKind::Missing,
        entry_count: None,
      });
    }
    Err(error) => return Err(error),
  };

  if !metadata.is_dir() {
    return Ok(PathDescription {
      kind: PathKind::File,
      entry_count: None,
    });
  }

  Ok(PathDescription {
    kind: PathKind::Directory,
    entry_count: count_entries(target),
  })
}

/// Counts what a directory lists, or answers `None` where it cannot be listed at all.
fn count_entries(directory: &Path) -> Option<u64> {
  Some(fs::read_dir(directory).ok()?.flatten().count() as u64)
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::path::PathBuf;

  use super::*;

  /// A directory of this test's own under `target`, emptied first so a re-run counts what this run created.
  fn root(name: &str) -> PathBuf {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../../target/tests/describe-path")
      .join(name);

    if path.exists() {
      fs::remove_dir_all(&path).expect("a removable test root");
    }

    fs::create_dir_all(&path).expect("a test root");

    path
  }

  #[test]
  fn a_missing_path_is_described_rather_than_refused() {
    let described: PathDescription = describe(&root("missing").join("nothing")).expect("a description");

    assert_eq!(described.kind, PathKind::Missing);
    assert_eq!(described.entry_count, None);
  }

  #[test]
  fn a_file_is_not_counted() {
    let file: PathBuf = root("file").join("system.ltx");

    File::create(&file).expect("a file");

    let described: PathDescription = describe(&file).expect("a description");

    assert_eq!(described.kind, PathKind::File);
    assert_eq!(described.entry_count, None);
  }

  #[test]
  fn only_immediate_entries_are_counted() {
    let directory: PathBuf = root("immediate");
    let nested: PathBuf = directory.join("nested");

    File::create(directory.join("a.ltx")).expect("a file");
    File::create(directory.join("b.ltx")).expect("a file");
    fs::create_dir(&nested).expect("a directory");
    File::create(nested.join("c.ltx")).expect("a file");

    let described: PathDescription = describe(&directory).expect("a description");

    assert_eq!(described.kind, PathKind::Directory);
    assert_eq!(described.entry_count, Some(3));
  }

  #[test]
  fn an_empty_directory_is_not_a_missing_one() {
    let described: PathDescription = describe(&root("empty")).expect("a description");

    assert_eq!(described.kind, PathKind::Directory);
    assert_eq!(described.entry_count, Some(0));
  }
}
