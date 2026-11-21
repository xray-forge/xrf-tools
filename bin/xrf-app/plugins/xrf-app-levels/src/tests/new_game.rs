use std::collections::HashMap;
use std::fs;

use xrf_dltx::DltxDialect;
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_material::fixtures::FixtureTree;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::level::LevelConfigs;
use crate::spawn::LevelSpawnRelease;
use crate::tests::fixtures::mount_search;

fn write(tree: &FixtureTree, path: &str, text: &str) {
  let path = tree.root().join(path);

  fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("directory is creatable");
  fs::write(path, text).expect("config is writable");
}

fn released(tree: &FixtureTree, is_dltx: bool) -> HashMap<String, LevelSpawnRelease> {
  let configs: LevelConfigs = LevelConfigs {
    roots: XrayRoots::one(tree.root().to_path_buf(), XrayMountMode::Directory),
    dialect: DltxDialect::select(is_dltx),
  };

  LevelSpawnRelease::read_new_game(&configs.open(&mount_search(tree)).expect("the configs open"))
}

#[test]
fn lists_the_names_a_section_keys_without_values() -> XrfResult {
  let ltx: Ltx = Ltx::read_from_str(
    "; objects to remove\n[remove_objects]\nesc_btr\nesc_zone_mine_field_soc_0000\n\n[other]\nkept = 1\n",
  )?;

  assert_eq!(
    LevelSpawnRelease::list_released(&ltx, "remove_objects"),
    vec!["esc_btr".to_owned(), "esc_zone_mine_field_soc_0000".to_owned()]
  );

  Ok(())
}

#[test]
fn lists_nothing_of_a_missing_section() -> XrfResult {
  let ltx: Ltx = Ltx::read_from_str("[settings]\nenabled = true\n")?;

  assert!(LevelSpawnRelease::list_released(&ltx, "replace_items").is_empty());

  Ok(())
}

#[test]
fn reads_the_lists_as_the_level_dialect_patches_them() {
  let tree: FixtureTree = FixtureTree::new("level_new_game_dltx");

  write(
    &tree,
    "configs/plugins/new_game_setup.ltx",
    "[remove_objects]\nesc_btr\nesc_trader\n",
  );
  write(
    &tree,
    "configs/plugins/mod_new_game_setup_test.ltx",
    "![remove_objects]\nesc_mod_added\n!esc_trader\n",
  );
  write(
    &tree,
    "configs/items/settings/dynamic_item_spawn.ltx",
    "[replace_items]\nesc_medkit\n",
  );

  let patched: HashMap<String, LevelSpawnRelease> = released(&tree, true);

  // As Anomaly's `ini_file` reads them: the mod adds one object and keeps another on the level.
  assert_eq!(patched.get("esc_btr"), Some(&LevelSpawnRelease::RemoveObjects));
  assert_eq!(patched.get("esc_mod_added"), Some(&LevelSpawnRelease::RemoveObjects));
  assert_eq!(patched.get("esc_trader"), None);
  assert_eq!(patched.get("esc_medkit"), Some(&LevelSpawnRelease::ReplaceItems));

  // Without the dialect a `mod_` file is just another file, and the list stays as written.
  let plain: HashMap<String, LevelSpawnRelease> = released(&tree, false);

  assert_eq!(plain.get("esc_trader"), Some(&LevelSpawnRelease::RemoveObjects));
  assert_eq!(plain.get("esc_mod_added"), None);
}

#[test]
fn releases_nothing_where_the_game_has_no_lists() {
  assert!(released(&FixtureTree::new("level_new_game_none"), true).is_empty());
}
