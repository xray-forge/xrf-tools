//! Narrowing the comparison, which must never invent or hide a difference by acting on one side only.

use std::path::PathBuf;

use crate::patch::config::{ArchivePatchConfig, ArchivePatchScope};
use crate::patch::tests::fixtures::{BASE_FILES, BINARY, CONFIG_EDITED, config, create_tree, destination, names_of};
use crate::patch::{ArchivePatchResult, ArchivePatcher};

/// A scope from authored prefixes, as the configuration would fold it.
fn scope(include: &[&str], ignore: &[&str], extensions: &[&str]) -> ArchivePatchScope {
  let owned = |values: &[&str]| values.iter().map(|value| String::from(*value)).collect::<Vec<_>>();

  ArchivePatchScope::new(&owned(include), &owned(ignore), &owned(extensions)).expect("the scope folds")
}

#[test]
fn an_empty_scope_admits_the_whole_world() {
  assert!(scope(&[], &[], &[]).admits("configs\\system.ltx"));
}

#[test]
fn an_include_admits_only_its_own_subtree() {
  let scope: ArchivePatchScope = scope(&["configs"], &[], &[]);

  assert!(scope.admits("configs\\system.ltx"));
  assert!(!scope.admits("textures\\wall.dds"));
  assert!(scope.is_narrowed());
}

#[test]
fn a_prefix_matches_on_component_boundaries() {
  let scope: ArchivePatchScope = scope(&["configs"], &[], &[]);

  assert!(
    !scope.admits("configs_backup\\system.ltx"),
    "a longer sibling is outside"
  );
}

#[test]
fn an_ignore_beats_an_include() {
  let scope: ArchivePatchScope = scope(&["configs"], &["configs\\weapons"], &[]);

  assert!(scope.admits("configs\\system.ltx"));
  assert!(!scope.admits("configs\\weapons\\ak74.ltx"));
}

#[test]
fn a_prefix_is_folded_to_the_engine_identity() {
  // Authored as a person types it; compared as the engine registers it.
  assert!(scope(&["Configs/Weapons"], &[], &[]).admits("configs\\weapons\\ak74.ltx"));
}

#[test]
fn an_excluded_extension_keeps_a_file_out() {
  let scope: ArchivePatchScope = scope(&[], &[], &["*.txt"]);

  assert!(!scope.admits("configs\\readme.txt"));
  assert!(scope.admits("configs\\system.ltx"));
}

#[test]
fn an_unaddressable_prefix_is_refused() {
  let traversal: Vec<String> = vec![String::from("configs\\..\\secrets")];

  assert!(ArchivePatchScope::new(&traversal, &[], &[]).is_err());
}

#[test]
fn a_scope_narrows_both_sides_alike() {
  let scope: &str = "patch_scope_narrows_both_sides";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(
    scope,
    "target",
    &[
      ("configs\\system.ltx", CONFIG_EDITED),
      ("configs\\weapons\\ak74.ltx", crate::patch::tests::fixtures::CONFIG),
      ("textures\\wall.dds", BINARY),
      ("textures\\floor.dds", BINARY),
    ],
  );
  let mut configured: ArchivePatchConfig = config(&base, &target, &destination(scope));

  configured.include = vec![String::from("configs")];

  let result: ArchivePatchResult = ArchivePatcher::compare(&configured).expect("a scoped run answers");

  assert_eq!(names_of(&result.modified), ["configs\\system.ltx"]);
  assert!(
    result.added.is_empty(),
    "the added texture is outside the scope on both sides, so it is not a difference"
  );
}

#[test]
fn an_ignored_prefix_is_dropped_from_both_sides() {
  let scope: &str = "patch_ignored_prefix_drops_both_sides";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(
    scope,
    "target",
    &[("configs\\system.ltx", CONFIG_EDITED), ("textures\\wall.dds", BINARY)],
  );
  let mut configured: ArchivePatchConfig = config(&base, &target, &destination(scope));

  configured.ignore = vec![String::from("configs\\weapons")];

  let result: ArchivePatchResult = ArchivePatcher::compare(&configured).expect("an ignoring run answers");

  assert_eq!(names_of(&result.modified), ["configs\\system.ltx"]);
  assert_eq!(
    result.added.len(),
    0,
    "the weapon config only the base holds is ignored on both sides"
  );
}
