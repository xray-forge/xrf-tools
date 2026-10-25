use std::fs::{self, File};
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::paths::PathDescription;
use crate::paths::path_description::PathKind;

/// A directory of this test's own among the generated test resources, emptied first so a re-run counts what this run
/// created.
fn root(name: &str) -> PathBuf {
  let path: PathBuf = build_absolute_generated_test_resource_path(&format!("describe-path/{name}"));

  if path.exists() {
    fs::remove_dir_all(&path).expect("a removable test root");
  }

  fs::create_dir_all(&path).expect("a test root");

  path
}

#[test]
fn a_missing_path_is_described_rather_than_refused() {
  let described: PathDescription = PathDescription::read(&root("missing").join("nothing")).expect("a description");

  assert_eq!(described.kind, PathKind::Missing);
  assert_eq!(described.entry_count, None);
}

#[test]
fn a_file_is_not_counted() {
  let file: PathBuf = root("file").join("system.ltx");

  File::create(&file).expect("a file");

  let described: PathDescription = PathDescription::read(&file).expect("a description");

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

  let described: PathDescription = PathDescription::read(&directory).expect("a description");

  assert_eq!(described.kind, PathKind::Directory);
  assert_eq!(described.entry_count, Some(3));
}

#[test]
fn an_empty_directory_is_not_a_missing_one() {
  let described: PathDescription = PathDescription::read(&root("empty")).expect("a description");

  assert_eq!(described.kind, PathKind::Directory);
  assert_eq!(described.entry_count, Some(0));
}
