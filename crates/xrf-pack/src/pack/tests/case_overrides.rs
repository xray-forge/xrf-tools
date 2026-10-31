//! A patch volume spelling a base file apart only by case: the engine folds the two names into one file and reads the
//! patch's, so every way out of the set must write the patch's too.

use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveProject;
use xrf_job::ExecutionRequest;
use xrf_test_utils::GeneratedResources;

use crate::pack::config::ArchivePackMode;
use crate::pack::tests::fixtures::{empty_destination, pack_volume, read_mounted};
use crate::unpack::{ArchiveExtractDirectoryResult, ArchiveUnpackResult, ArchiveUnpacker};

const BASE: &[u8] = b"base=1\n";
const PATCH: &[u8] = b"patch2\n";

/// The base and its case-only patch, as the directory holding both volumes, the patch's name sorting after the base's.
fn create_set(scope: &str) -> PathBuf {
  let scope: String = format!("case_overrides/{scope}");
  let destination: PathBuf = empty_destination(&scope);

  for (name, file) in [
    ("a_base", ("configs\\system.ltx", BASE)),
    ("z_patch", ("configs\\System.ltx", PATCH)),
  ] {
    pack_volume(&scope, name, &[file], |config| config.mode = ArchivePackMode::Store);
  }

  destination
}

/// The one file a run left in `configs`, with its bytes.
fn read_only_file(directory: &Path) -> Vec<u8> {
  let files: Vec<PathBuf> = fs::read_dir(directory)
    .expect("an extracted directory")
    .map(|entry| entry.expect("a directory entry").path())
    .collect();

  assert_eq!(files.len(), 1, "one file on every host: {files:?}");

  fs::read(&files[0]).expect("the extracted file")
}

#[test]
fn the_patch_is_what_the_set_mounts() {
  let set: PathBuf = create_set("mounts");

  assert_eq!(read_mounted(&set, "configs\\system.ltx").as_deref(), Some(PATCH));
}

#[test]
fn unpacking_and_extracting_write_the_patch_whatever_the_map_order() {
  let set: PathBuf = create_set("writes");

  // Each open draws a fresh map order, which used to decide the survivor about half the time.
  for (run, workers) in [1, 1, 2, 2, 4, 4, 8, 8].into_iter().enumerate() {
    let workers: NonZeroUsize = NonZeroUsize::new(workers).expect("a non-zero worker count");
    let project: ArchiveProject = ArchiveProject::new(&set).expect("the set opens");
    let unpacked: PathBuf = GeneratedResources::path(&format!("case_overrides/writes/unpacked-{run}"));
    let extracted: PathBuf = GeneratedResources::path(&format!("case_overrides/writes/extracted-{run}"));

    let _ = fs::remove_dir_all(&unpacked);
    let _ = fs::remove_dir_all(&extracted);

    let (unpack, extract): (ArchiveUnpackResult, ArchiveExtractDirectoryResult) = ExecutionRequest::Workers(workers)
      .resolve()
      .install(|| {
        (
          ArchiveUnpacker::unpack(&project, &unpacked).expect("the set unpacks"),
          ArchiveUnpacker::extract_directory(&project, "configs", &extracted).expect("the directory extracts"),
        )
      })
      .expect("the pool starts");

    assert_eq!(unpack.files_unpacked, 1, "run {run}");
    assert_eq!(extract.extracted_count, 1, "run {run}");
    assert_eq!(
      read_only_file(&unpacked.join("gamedata").join("configs")),
      PATCH,
      "run {run}"
    );
    assert_eq!(read_only_file(&extracted), PATCH, "run {run}");
  }
}
