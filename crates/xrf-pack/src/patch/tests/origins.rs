//! What the report says about where entries came from, and why it says it once.

use std::path::{Path, PathBuf};

use crate::patch::ArchivePatchResult;
use crate::patch::compare::{ArchivePatchChange, ArchivePatchOrigin, ArchivePatchSide};
use crate::patch::tests::fixtures::{
  BASE_FILES, BINARY, CONFIG, CONFIG_EDITED, compare, create_tree, create_volumes, destination,
};

/// The origin a side names, resolved against the report's table.
fn origin_of(result: &ArchivePatchResult, side: &ArchivePatchSide) -> ArchivePatchOrigin {
  result
    .origins
    .get(side.origin as usize)
    .expect("every index names a listed origin")
    .clone()
}

/// The path an origin points at, whichever kind it is.
fn path_of(origin: &ArchivePatchOrigin) -> PathBuf {
  match origin {
    ArchivePatchOrigin::Directory { root } => root.clone(),
    ArchivePatchOrigin::Archive { path } => path.clone(),
  }
}

#[test]
fn two_loose_trees_are_named_twice_rather_than_once_per_entry() {
  // The whole point of the table. Every side of every change resolves to one of two roots, so listing the root on
  // each of them is the bulk of a large report and says nothing this does not.
  let scope: &str = "patch_origins_two_loose_trees";
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

  assert_eq!(
    result.origins.len(),
    2,
    "one entry per root, however many changes there are"
  );

  let listed: Vec<PathBuf> = result.origins.iter().map(path_of).collect();

  assert!(listed.contains(&base), "the base root is listed");
  assert!(listed.contains(&target), "the target root is listed");
}

#[test]
fn every_side_resolves_to_the_root_it_was_actually_read_from() {
  // An index is only worth writing if it means the right thing, and base and target share one table: a side numbered
  // against the wrong half would still resolve, just to a lie.
  let scope: &str = "patch_origins_resolve_per_side";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  let modified: &ArchivePatchChange = result.modified.first().expect("the edited config is modified");

  assert_eq!(
    path_of(&origin_of(&result, modified.base.as_ref().expect("a base side"))),
    base
  );
  assert_eq!(
    path_of(&origin_of(&result, modified.target.as_ref().expect("a target side"))),
    target
  );
}

#[test]
fn an_archived_side_is_listed_as_its_volume_set_rather_than_as_a_directory() {
  // The kind survives interning, because "which volume won" is the question an installation makes worth asking.
  let scope: &str = "patch_origins_name_the_volume_set";
  let base: PathBuf = create_volumes(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(
    scope,
    "target",
    &[("configs\\system.ltx", CONFIG_EDITED), ("textures\\wall.dds", BINARY)],
  );
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  let modified: &ArchivePatchChange = result.modified.first().expect("the edited config is modified");
  let from_base: ArchivePatchOrigin = origin_of(&result, modified.base.as_ref().expect("a base side"));
  let from_target: ArchivePatchOrigin = origin_of(&result, modified.target.as_ref().expect("a target side"));

  assert!(
    matches!(from_base, ArchivePatchOrigin::Archive { .. }),
    "the packed side is named as an archive"
  );
  assert!(
    matches!(from_target, ArchivePatchOrigin::Directory { ref root } if root == &target),
    "the loose side is named as its root"
  );
}

#[test]
fn a_run_over_one_world_lists_that_world_once() {
  // Both sides sharing one table is what makes this one entry rather than two identical ones.
  let scope: &str = "patch_origins_shared_between_sides";
  let files: &[(&str, &[u8])] = &[("configs\\system.ltx", CONFIG)];
  let base: PathBuf = create_tree(scope, "base", files);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let result: ArchivePatchResult = compare(&base, &target, &destination(scope));

  let roots: Vec<&Path> = result
    .origins
    .iter()
    .map(|origin| match origin {
      ArchivePatchOrigin::Directory { root } => root.as_path(),
      ArchivePatchOrigin::Archive { path } => path.as_path(),
    })
    .collect();

  assert_eq!(roots.len(), 2);
  assert_ne!(roots[0], roots[1], "no root is listed twice");
}
