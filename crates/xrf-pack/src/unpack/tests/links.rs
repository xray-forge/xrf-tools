//! Containment of archive-controlled names against a destination tree that already holds links.
//!
//! Rejecting `..` in an entry name proves nothing here: every name below is perfectly ordinary, and the escape is a
//! link the destination already contained. Each test therefore puts a sentinel outside the destination and asserts
//! nothing reached it, rather than only that the call failed.

use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveProject;

use std::ffi::OsStr;

use xrf_error::XrfError;

use super::fixtures::{Entry, create_project, create_temporary_directory, link_directory, link_file};
use crate::unpack::rooted_destination::RootedDestination;
use crate::{ArchiveUnpackOptions, ArchiveUnpacker};

const ONE: NonZeroUsize = NonZeroUsize::new(1).expect("a non-zero worker count");

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

#[test]
fn unpack_refuses_a_directory_link_below_the_destination() {
  let directory: PathBuf = create_temporary_directory("link-unpack-directory");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let sentinel: PathBuf = create_sentinel(&directory);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(&out).expect("destination");

  // The destination is a tree the user already has, and one directory in it points elsewhere.
  if !link_directory(&sentinel, &out.join("configs")) {
    return;
  }

  assert!(
    ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(ONE)).is_err(),
    "unpacking through a link must fail rather than follow it"
  );
  assert_sentinel_untouched(&sentinel);
}

#[test]
fn unpack_refuses_a_file_link_in_place_of_an_entry() {
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
    ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(ONE)).is_err(),
    "writing an entry over a link must fail rather than follow it"
  );
  assert_sentinel_untouched(&sentinel);
}

#[test]
fn extract_directory_refuses_a_directory_link_below_the_destination() {
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
    ArchiveUnpacker::extract_directory(&project, "configs", &out).is_err(),
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

/// The containment walk creates and verifies one component at a time, so an ordinary tree that is already there has to
/// keep unpacking rather than be mistaken for something to refuse.
#[test]
fn unpack_writes_into_a_destination_tree_that_already_exists() {
  let directory: PathBuf = create_temporary_directory("link-existing-tree");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let out: PathBuf = directory.join("out");

  fs::create_dir_all(out.join("configs")).expect("destination");
  fs::write(out.join("configs").join("system.ltx"), b"stale").expect("previous unpack");

  ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(ONE)).expect("unpack");

  assert_eq!(
    fs::read_to_string(out.join("configs").join("system.ltx")).expect("written file"),
    "[section]"
  );
}

/// A parent handed in from outside the root is refused rather than trusted.
///
/// `create_file_in` skips the walk down because its caller already verified the parent this run. That shortcut is only
/// sound while the type still owns containment, so the one thing it must not do is accept any path a caller offers.
#[test]
fn create_file_in_refuses_a_parent_outside_the_root() {
  let directory: PathBuf = create_temporary_directory("rooted-parent-outside");
  let root: PathBuf = directory.join("out");
  let outside: PathBuf = directory.join("elsewhere");

  fs::create_dir_all(&outside).expect("a directory beside the root");

  let destination: RootedDestination = RootedDestination::new(&root);

  destination.create_root().expect("the root is created once");

  let error: XrfError = destination
    .create_file_in(&outside, OsStr::new("system.ltx"))
    .expect_err("a parent outside the root is refused");

  assert!(matches!(error, XrfError::Invalid { .. }), "{error}");
  assert!(
    !outside.join("system.ltx").exists(),
    "a refused parent must not have been written into"
  );

  // The root itself is a parent this type owns, so the same call through it succeeds.
  destination
    .create_file_in(&root, OsStr::new("system.ltx"))
    .expect("a parent below the root is accepted");

  assert!(root.join("system.ltx").is_file());
}
