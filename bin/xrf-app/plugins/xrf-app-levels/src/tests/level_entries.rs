use std::fs;
use std::path::PathBuf;

use xrf_level::{LevelFile, LevelGeomFile};
use xrf_material::fixtures::FixtureTree;

use crate::level::LevelEntry;
use crate::tests::fixtures::mount_search;

fn touch(tree: &FixtureTree, path: &str) {
  let file: PathBuf = tree.root().join(path);

  fs::create_dir_all(file.parent().expect("a file has a directory")).expect("directory created");
  fs::write(file, []).expect("file written");
}

// A picker offers a directory only for a level it can open, and says before it is opened whether it draws anything.
#[test]
fn lists_each_directory_holding_a_bundle_and_whether_it_draws() {
  let tree: FixtureTree = FixtureTree::new("level_entries");

  touch(&tree, &format!("levels/zaton/{}", LevelFile::FILE_NAME));
  touch(&tree, &format!("levels/zaton/{}", LevelGeomFile::FILE_NAME));
  touch(&tree, &format!("levels/lmaps/{}", LevelFile::FILE_NAME));
  touch(&tree, "levels/textures/terrain.dds");

  let mut entries: Vec<LevelEntry> = LevelEntry::list(&mount_search(&tree)).expect("the levels list");

  entries.sort_by(|left, right| left.name.cmp(&right.name));

  assert_eq!(
    entries,
    [
      LevelEntry {
        has_geometry: false,
        logical_path: String::from("levels\\lmaps"),
        name: String::from("lmaps"),
      },
      LevelEntry {
        has_geometry: true,
        logical_path: String::from("levels\\zaton"),
        name: String::from("zaton"),
      },
    ]
  );
}
