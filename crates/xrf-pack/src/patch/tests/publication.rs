//! What the two writing doors leave on disk, and what the report says became of the difference.

use std::path::PathBuf;

use crate::patch::tests::fixtures::{
  BASE_FILES, BINARY, CONFIG, CONFIG_EDITED, compare, create_tree, destination, patch, published_bytes, published_names,
};
use crate::patch::{ArchivePatchPublication, ArchivePatchResult};

#[test]
fn carries_only_the_changed_entries_into_the_published_patch() {
  let scope: &str = "patch_carries_only_the_changed_entries";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(
    scope,
    "target",
    &[
      ("configs\\system.ltx", CONFIG_EDITED),
      ("configs\\weapons\\ak74.ltx", CONFIG),
      ("textures\\wall.dds", BINARY),
      ("textures\\floor.dds", BINARY),
    ],
  );
  let into: PathBuf = destination(scope);
  let result: ArchivePatchResult = patch(&base, &target, &into);

  assert_eq!(
    published_names(&into),
    ["configs\\system.ltx", "textures\\floor.dds"],
    "the patch holds the modified and the added entry and nothing else"
  );
  assert_eq!(
    published_bytes(&into, "configs\\system.ltx"),
    CONFIG_EDITED,
    "a modified entry is carried from the target"
  );
  assert_eq!(result.get_carried_count(), 2);
  assert!(result.publication.is_published());
}

#[test]
fn a_comparison_reports_the_difference_and_writes_nothing() {
  let scope: &str = "patch_comparison_writes_nothing";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let into: PathBuf = destination(scope);
  let result: ArchivePatchResult = compare(&base, &target, &into);

  assert!(matches!(result.publication, ArchivePatchPublication::Compared));
  assert!(!result.modified.is_empty(), "it still says what differs");
  assert!(!into.exists(), "a comparison does not even create the destination");
}

#[test]
fn publishing_two_worlds_that_agree_writes_no_volume() {
  // Distinct from a comparison: this door would have written, and there was nothing worth writing.
  let scope: &str = "patch_agreeing_worlds_write_no_volume";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", BASE_FILES);
  let into: PathBuf = destination(scope);
  let result: ArchivePatchResult = patch(&base, &target, &into);

  assert!(matches!(result.publication, ArchivePatchPublication::Unnecessary));
  assert!(!into.exists());
}

#[test]
fn the_published_set_reports_what_the_packer_saw() {
  let scope: &str = "patch_published_set_reports_the_pack";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(
    scope,
    "target",
    &[("configs\\system.ltx", CONFIG_EDITED), ("textures\\wall.dds", BINARY)],
  );
  let into: PathBuf = destination(scope);
  let result: ArchivePatchResult = patch(&base, &target, &into);
  let published = result.publication.get_published().expect("a volume set was written");

  assert_eq!(published.volumes.len(), 1);
  assert_eq!(
    published.files_compressed + published.files_stored,
    result.get_carried_count(),
    "every carried entry is accounted for by the packer's own counts"
  );
  assert!(published.size_written > 0);
}

#[test]
fn a_patch_reads_back_through_the_engines_own_identity() {
  // The whole point of the volume: the engine folds both spellings to one name and the patch registers under it.
  let scope: &str = "patch_reads_back_by_engine_identity";
  let base: PathBuf = create_tree(scope, "base", &[("Configs\\System.ltx", CONFIG)]);
  let target: PathBuf = create_tree(scope, "target", &[("Configs\\System.ltx", CONFIG_EDITED)]);
  let into: PathBuf = destination(scope);

  patch(&base, &target, &into);

  assert_eq!(published_names(&into), ["configs\\system.ltx"]);
  assert_eq!(published_bytes(&into, "configs\\system.ltx"), CONFIG_EDITED);
}
