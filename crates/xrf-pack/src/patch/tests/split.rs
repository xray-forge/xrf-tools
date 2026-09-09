//! One installation supplying both sides: its volumes against its own loose tree.

use std::path::PathBuf;

use crate::patch::config::ArchivePatchShape;
use crate::patch::tests::fixtures::{
  BASE_FILES, BINARY, CONFIG, CONFIG_EDITED, compare_split, create_installation, create_tree, destination, names_of,
  patch_split, published_bytes, published_names, split_config,
};
use crate::patch::{ArchivePatchResult, ArchivePatcher};

#[test]
fn an_installation_compares_its_volumes_against_its_own_loose_tree() {
  // The case no pair of paths can pose. Pointing both sides at the installation finds every loose file equal to
  // itself, because the loose tree wins inside the merged world; naming `db\` reaches only the volumes directly in it.
  let scope: &str = "patch_split_volumes_against_loose";
  let install: PathBuf = create_installation(
    scope,
    "game",
    BASE_FILES,
    &[
      ("configs\\system.ltx", CONFIG_EDITED),
      ("configs\\weapons\\abakan.ltx", CONFIG),
    ],
  );
  let result: ArchivePatchResult = compare_split(&install, &destination(scope));

  assert_eq!(
    names_of(&result.modified),
    ["configs\\system.ltx"],
    "the loose file that differs from its archived counterpart"
  );
  assert_eq!(
    names_of(&result.added),
    ["configs\\weapons\\abakan.ltx"],
    "the loose file no volume holds"
  );
  assert!(
    result.removed.is_empty(),
    "the rest of the release is untouched, not deleted"
  );
}

#[test]
fn a_loose_file_matching_its_archived_counterpart_is_not_carried() {
  // The pruning the workflow is worth having for: a modder copies a directory, edits one file, and ships one file.
  let scope: &str = "patch_split_prunes_unchanged_copies";
  let install: PathBuf = create_installation(
    scope,
    "game",
    BASE_FILES,
    &[
      ("configs\\system.ltx", CONFIG_EDITED),
      // Copied out of the archive and never edited.
      ("configs\\weapons\\ak74.ltx", CONFIG),
      ("textures\\wall.dds", BINARY),
    ],
  );
  let into: PathBuf = destination(scope);
  let result: ArchivePatchResult = patch_split(&install, &into);

  assert_eq!(result.unchanged, 2, "the two untouched copies decided themselves out");
  assert_eq!(published_names(&into), ["configs\\system.ltx"]);
  assert_eq!(published_bytes(&into, "configs\\system.ltx"), CONFIG_EDITED);
}

#[test]
fn an_installation_with_nothing_loose_says_so_rather_than_blaming_the_path() {
  // A fresh install is not a typo, and the generic empty-side message would send a modder looking for one.
  let scope: &str = "patch_split_nothing_loose";
  let install: PathBuf = create_installation(scope, "game", BASE_FILES, &[]);
  let message: String = ArchivePatcher::compare(&split_config(&install, &destination(scope)))
    .expect_err("an installation with an empty loose tree is refused")
    .to_string();

  assert!(
    message.contains("holds no file to compare"),
    "'{message}' says the loose tree is empty"
  );
  assert!(
    message.contains("gamedata"),
    "'{message}' says where the changed files go"
  );
}

#[test]
fn release_shape_is_refused_for_a_split_input() {
  // Two halves of one installation are not two releases, so "what did the target drop" has no answer here.
  let scope: &str = "patch_split_refuses_release_shape";
  let install: PathBuf = create_installation(scope, "game", BASE_FILES, &[("configs\\system.ltx", CONFIG_EDITED)]);
  let configured = split_config(&install, &destination(scope)).with_shape(ArchivePatchShape::Release);
  let message: String = ArchivePatcher::compare(&configured)
    .expect_err("release shape needs two releases")
    .to_string();

  assert!(
    message.contains("not two releases"),
    "'{message}' says why the shape does not apply"
  );
}

#[test]
fn a_destination_inside_the_game_is_named_as_the_game() {
  // `db\patches\` is where a patch is deployed, so it is the first place someone points the output at. Both halves of
  // a split carry the same root, and calling it "the base root" would name a directory the caller never typed.
  let scope: &str = "patch_split_destination_inside_the_game";
  let install: PathBuf = create_installation(scope, "game", BASE_FILES, &[("configs\\system.ltx", CONFIG_EDITED)]);
  let message: String = ArchivePatcher::patch(&split_config(&install, &install.join("db").join("patches")))
    .expect_err("a patch cannot be written into the tree it compares")
    .to_string();

  assert!(
    message.contains("is inside the game"),
    "'{message}' names the game rather than a role"
  );
}

#[test]
fn a_gamedata_tree_named_as_the_input_says_it_is_not_a_game() {
  // The mirror mistake: pointing at the mod folder rather than at the game. Its files have nothing to override, and
  // the generic empty-mount message would call that a missing root.
  let scope: &str = "patch_split_input_without_volumes";
  let loose: PathBuf = create_tree(scope, "modfolder", BASE_FILES);
  let message: String = ArchivePatcher::compare(&split_config(&loose, &destination(scope)))
    .expect_err("a tree with no volumes cannot supply both sides")
    .to_string();

  assert!(
    message.contains("holds no archive volumes"),
    "'{message}' says what the input is missing"
  );
}
