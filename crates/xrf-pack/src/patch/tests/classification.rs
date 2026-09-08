//! Which class the comparison puts each entry in, and what deciding it cost.

use std::path::PathBuf;

use crate::patch::ArchivePatchResult;
use crate::patch::compare::ArchivePatchClass;
use crate::patch::tests::fixtures::{
  BASE_FILES, BINARY, CONFIG, CONFIG_EDITED, CONFIG_LONGER, compare, create_tree, create_volumes, destination, names_of,
};

#[test]
fn classifies_added_modified_and_removed_between_two_loose_trees() {
  let scope: &str = "patch_classifies_between_two_loose_trees";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(
    scope,
    "target",
    &[
      ("configs\\system.ltx", CONFIG_EDITED),
      ("configs\\weapons\\ak74.ltx", CONFIG),
      ("configs\\weapons\\abakan.ltx", CONFIG),
    ],
  );
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  assert_eq!(names_of(&result.added), ["configs\\weapons\\abakan.ltx"]);
  assert_eq!(names_of(&result.modified), ["configs\\system.ltx"]);
  assert_eq!(names_of(&result.removed), ["textures\\wall.dds"]);
  assert_eq!(result.unchanged, 1, "the untouched weapon config is not carried");
}

#[test]
fn two_archived_sides_decide_without_reading_a_payload() {
  // The cheap shape, and the reason the recorded checksum is published at all: two name tables answer everything.
  let scope: &str = "patch_archived_sides_read_no_payload";
  let base: PathBuf = create_volumes(scope, "base", BASE_FILES);
  let target: PathBuf = create_volumes(
    scope,
    "target",
    &[
      ("configs\\system.ltx", CONFIG_EDITED),
      ("configs\\weapons\\ak74.ltx", CONFIG),
      ("textures\\wall.dds", BINARY),
    ],
  );
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  assert_eq!(names_of(&result.modified), ["configs\\system.ltx"]);
  assert_eq!(result.unchanged, 2);
  assert_eq!(
    result.payloads_read, 0,
    "an archive records the checksum, so nothing had to be decompressed to compare"
  );
}

#[test]
fn a_pair_differing_in_size_is_decided_without_a_checksum() {
  let scope: &str = "patch_size_settles_a_pair";
  let base: PathBuf = create_tree(scope, "base", &[("configs\\system.ltx", CONFIG)]);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_LONGER)]);
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  assert_eq!(names_of(&result.modified), ["configs\\system.ltx"]);
  assert_eq!(
    result.payloads_read, 0,
    "two loose files of different length never need to be read"
  );
}

#[test]
fn a_loose_pair_of_equal_size_is_read_to_decide() {
  let scope: &str = "patch_equal_size_loose_pair_is_read";
  let base: PathBuf = create_tree(scope, "base", &[("configs\\system.ltx", CONFIG)]);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  assert_eq!(names_of(&result.modified), ["configs\\system.ltx"]);
  assert_eq!(result.payloads_read, 1, "neither loose side records a checksum");
}

#[test]
fn an_archived_base_hashes_only_the_loose_target() {
  // The release workflow: last release as volumes, new build as a tree. Only the tree side costs a read.
  let scope: &str = "patch_archived_base_hashes_only_the_target";
  let base: PathBuf = create_volumes(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", BASE_FILES);
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  assert!(
    result.is_empty(),
    "the same content packed and loose is not a difference"
  );
  assert_eq!(result.payloads_read, BASE_FILES.len());
}

#[test]
fn two_identical_worlds_carry_nothing_and_succeed() {
  // An empty difference is a true answer: two releases may genuinely agree, and that is not a failure.
  let scope: &str = "patch_identical_worlds_carry_nothing";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", BASE_FILES);
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  assert!(result.is_empty());
  assert_eq!(result.unchanged, BASE_FILES.len());
  assert!(result.removed.is_empty());
}

#[test]
fn every_change_agrees_with_the_class_it_reports() {
  let scope: &str = "patch_classes_agree_with_their_lists";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(
    scope,
    "target",
    &[("configs\\system.ltx", CONFIG_EDITED), ("textures\\floor.dds", BINARY)],
  );
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  for change in &result.added {
    assert_eq!(change.class, ArchivePatchClass::Added);
    assert!(change.base.is_none(), "an added entry has no base side");
    assert!(change.target.is_some());
  }

  for change in &result.modified {
    assert_eq!(change.class, ArchivePatchClass::Modified);
    assert!(change.base.is_some() && change.target.is_some());
  }

  for change in &result.removed {
    assert_eq!(change.class, ArchivePatchClass::Removed);
    assert!(change.target.is_none(), "a removed entry has no target side");
    assert!(change.base.is_some());
  }
}

#[test]
fn a_later_root_overrides_an_earlier_one() {
  // Engine order: `fsgame.ltx` declares the overriding path last, so the last root named wins.
  let scope: &str = "patch_later_root_overrides_earlier";
  let release: PathBuf = create_volumes(scope, "release", BASE_FILES);
  let overlay: PathBuf = create_tree(scope, "overlay", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);

  let layered =
    crate::patch::config::ArchivePatchConfig::new(vec![release, overlay], vec![target], destination(scope), "patch");
  let result: ArchivePatchResult =
    crate::patch::ArchivePatcher::compare(&layered).expect("a layered comparison answers");

  assert!(
    result.modified.is_empty(),
    "the overlay already carries the edit, so the target matches the base"
  );
}
