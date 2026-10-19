//! Extraction into a destination tree that already holds links.
//!
//! Every name below is ordinary, so what decides where a write lands is the link and the run's link policy. Each test
//! puts a sentinel outside the destination and asserts what reached it, rather than only whether the call failed.

use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveProject;
use xrf_test_utils::links::{link_directory, link_file};
use xrf_utils::HostLinkPolicy;

use super::fixtures::{Entry, create_project, create_temporary_directory, unpack_with};
use crate::{ArchiveExtractOptions, ArchiveUnpacker};

const ONE: NonZeroUsize = NonZeroUsize::new(1).expect("a non-zero worker count");

fn refusing_links() -> ArchiveExtractOptions {
  ArchiveExtractOptions::default().with_link_policy(HostLinkPolicy::Refuse)
}

/// A directory outside the destination, holding a file no extraction is allowed to touch.
fn create_sentinel(directory: &Path) -> PathBuf {
  let sentinel: PathBuf = directory.join("sentinel");

  fs::create_dir_all(&sentinel).expect("sentinel directory");
  fs::write(sentinel.join("keep.txt"), b"original").expect("sentinel file");

  sentinel
}

fn assert_sentinel_untouched(sentinel: &Path) {
  assert!(
    !sentinel.join("system.ltx").exists(),
    "an archive entry was written outside the destination"
  );
  assert_eq!(
    fs::read_to_string(sentinel.join("keep.txt")).expect("sentinel file"),
    "original",
    "an archive entry overwrote a file outside the destination"
  );
}

/// A junctioned or symlink-deployed folder is the user's layout, so the default unpack writes where it points.
#[test]
fn unpack_follows_a_directory_link_below_the_destination_by_default() {
  let directory: PathBuf = create_temporary_directory("link-follow-directory");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let sentinel: PathBuf = create_sentinel(&directory);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(&out).expect("destination");

  if !link_directory(&sentinel, &out.join("configs")) {
    return;
  }

  unpack_with(ONE, &project, &out).expect("unpack through the user's link");

  assert_eq!(
    fs::read_to_string(sentinel.join("system.ltx")).expect("written file"),
    "[section]"
  );
}

#[test]
fn unpack_refuses_a_directory_link_below_the_destination_when_asked() {
  let directory: PathBuf = create_temporary_directory("link-unpack-directory");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let sentinel: PathBuf = create_sentinel(&directory);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(&out).expect("destination");

  if !link_directory(&sentinel, &out.join("configs")) {
    return;
  }

  assert!(
    ArchiveUnpacker::unpack_opt(&project, &out, refusing_links()).is_err(),
    "unpacking through a link must fail rather than follow it"
  );
  assert_sentinel_untouched(&sentinel);
}

#[test]
fn unpack_refuses_a_file_link_in_place_of_an_entry_when_asked() {
  let directory: PathBuf = create_temporary_directory("link-unpack-file");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\keep.txt", b"[section]")]);
  let sentinel: PathBuf = create_sentinel(&directory);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(out.join("configs")).expect("destination");

  // Every directory on the way down is real; only the entry itself has been replaced by a link.
  if !link_file(&sentinel.join("keep.txt"), &out.join("configs").join("keep.txt")) {
    return;
  }

  assert!(
    ArchiveUnpacker::unpack_opt(&project, &out, refusing_links()).is_err(),
    "writing an entry over a link must fail rather than follow it"
  );
  assert_sentinel_untouched(&sentinel);
}

#[test]
fn extract_directory_refuses_a_directory_link_below_the_destination_when_asked() {
  let directory: PathBuf = create_temporary_directory("link-extract-directory");
  let project: ArchiveProject = create_project(
    &directory,
    &[Entry::stored("configs\\gameplay\\dialogs.xml", b"<game_dialogs/>")],
  );
  let sentinel: PathBuf = create_sentinel(&directory);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(&out).expect("destination");

  // The prefix is stripped, so `gameplay` is the first component written below the destination.
  if !link_directory(&sentinel, &out.join("gameplay")) {
    return;
  }

  assert!(
    ArchiveUnpacker::extract_directory_opt(&project, "configs", &out, refusing_links()).is_err(),
    "extracting through a link must fail rather than follow it"
  );
  assert!(
    !sentinel.join("dialogs.xml").exists(),
    "an archive entry was written outside the destination"
  );
  assert_sentinel_untouched(&sentinel);
}

/// The destination the caller named is theirs, links and all: only what an archive lays out inside it is constrained.
#[test]
fn extract_file_still_writes_to_the_linked_path_it_is_given() {
  let directory: PathBuf = create_temporary_directory("link-extract-file");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let sentinel: PathBuf = create_sentinel(&directory);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(&out).expect("destination");

  if !link_file(&sentinel.join("keep.txt"), &out.join("chosen.ltx")) {
    return;
  }

  ArchiveUnpacker::extract_file(&project, "configs\\system.ltx", out.join("chosen.ltx")).expect("extraction");

  assert_eq!(
    fs::read_to_string(sentinel.join("keep.txt")).expect("sentinel file"),
    "[section]"
  );
}

/// An ordinary tree that is already there keeps unpacking under the strict policy too.
#[test]
fn unpack_writes_into_a_destination_tree_that_already_exists() {
  let directory: PathBuf = create_temporary_directory("link-existing-tree");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(out.join("configs")).expect("destination");
  fs::write(out.join("configs").join("system.ltx"), b"stale").expect("previous unpack");

  ArchiveUnpacker::unpack_opt(&project, &out, refusing_links()).expect("unpack");

  assert_eq!(
    fs::read_to_string(out.join("configs").join("system.ltx")).expect("written file"),
    "[section]"
  );
}

/// An entry name that climbs out is refused before anything is written, under either policy.
#[test]
fn unpack_refuses_an_entry_name_that_leaves_the_destination() {
  for (index, name) in [
    "..\\system.ltx",
    "configs/../../system.ltx",
    "C:\\system.ltx",
    "C:system.ltx",
  ]
  .into_iter()
  .enumerate()
  {
    let directory: PathBuf = create_temporary_directory(&format!("escaping-entry-{index}"));
    let project: ArchiveProject = create_project(&directory, &[Entry::stored(name, b"[section]")]);
    let out: PathBuf = directory.join("out");

    assert!(unpack_with(ONE, &project, &out).is_err(), "'{name}' must be refused");
    assert!(
      !directory.join("system.ltx").exists(),
      "'{name}' was written outside the destination"
    );
  }
}
