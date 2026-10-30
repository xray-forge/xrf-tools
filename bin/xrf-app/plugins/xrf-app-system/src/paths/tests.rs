use std::fs::{self, File};
use std::path::PathBuf;

use xrf_test_utils::GeneratedResources;

use crate::paths::description::PathKind;
use crate::paths::{AppStorageKind, AppStorageUsage, PathDescription};

/// A directory of this test's own among the generated test resources, emptied first so a re-run counts what this run
/// created.
fn root(name: &str) -> PathBuf {
  let path: PathBuf = GeneratedResources::path(&format!("describe-path/{name}"));

  if path.exists() {
    fs::remove_dir_all(&path).expect("a removable test root");
  }

  fs::create_dir_all(&path).expect("a test root");

  path
}

/// Every file of the application's data directory is measured into its kind, by where it lies; only the renderer's
/// cache may be cleared.
#[test]
fn the_application_storage_is_measured_by_kind() {
  let root: PathBuf = root("storage");

  for (path, bytes) in [
    ("cache/renderer/vulkan_pipelines", 100),
    ("cache/renderer/shaders/a.bin", 50),
    ("cache/renderer/shaders/b.bin", 20),
    ("cache/thumbnails/a.png", 7),
    ("cache/.tidied", 0),
    ("cache/.lock", 0),
    ("logs/xrf-app.log", 3),
    ("preferences.json", 2),
    ("level-viewer/screenshots/a.png", 40),
    ("level-viewer/bursts/one/frame.png", 60),
    ("notes.txt", 1),
  ] {
    let file: PathBuf = root.join(path);

    fs::create_dir_all(file.parent().unwrap()).expect("a folder");
    fs::write(&file, vec![0u8; bytes]).expect("a file");
  }

  let usage: AppStorageUsage = AppStorageUsage::measure(&root);
  let measured: Vec<(AppStorageKind, u64, u64)> = usage
    .areas
    .iter()
    .map(|area| (area.kind, area.bytes, area.files))
    .collect();

  assert_eq!(
    measured,
    [
      (AppStorageKind::RendererCache, 170, 3),
      (AppStorageKind::OtherCaches, 7, 1),
      (AppStorageKind::Logs, 3, 1),
      (AppStorageKind::Preferences, 2, 1),
      (AppStorageKind::SavedFiles, 100, 2),
      (AppStorageKind::Other, 1, 1),
    ]
  );
  assert_eq!(usage.bytes, 283);
  assert!(usage.areas.iter().all(|area| area.last_used.is_some()));
  assert!(
    AppStorageKind::ALL
      .iter()
      .all(|kind| kind.get_clearable_folder().is_some() == (*kind == AppStorageKind::RendererCache))
  );
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
