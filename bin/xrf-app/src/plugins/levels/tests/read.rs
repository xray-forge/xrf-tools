use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::plugins::levels::read::to_level_logical_path;
use crate::plugins::levels::state::LevelSource;

fn zaton() -> LevelSource {
  LevelSource::Asset {
    logical_path: String::from("levels\\zaton"),
  }
}

#[test]
fn names_a_level_file_by_its_logical_path_inside_the_level() {
  assert_eq!(
    to_level_logical_path(&zaton(), "level.geom").unwrap().as_str(),
    "levels\\zaton\\level.geom"
  );
  assert_eq!(
    to_level_logical_path(&zaton(), "motions/door.anm").unwrap().as_str(),
    "levels\\zaton\\motions\\door.anm"
  );
}

// A spawn names an object's motion by a path its author wrote, which must not reach outside the level.
#[test]
fn refuses_a_name_leaving_the_level() {
  for file in [
    "",
    "../level.geom",
    "motions/../../secret",
    "/etc/passwd",
    "./level.geom",
    "..\\level.geom",
    "C:\\Windows\\win.ini",
    "\\level.geom",
  ] {
    assert!(to_level_logical_path(&zaton(), file).is_err(), "'{file}' was taken");
  }
}

// A directory is read through the search by the path its implied root gives it, so its reads are recorded too.
#[test]
fn names_a_level_directorys_files_by_the_root_it_lies_under() {
  let root: PathBuf = build_absolute_generated_test_resource_path("level_read/directory");
  let level: PathBuf = root.join("levels").join("l01");
  let stray: PathBuf = build_absolute_generated_test_resource_path("level_read/stray").join("l01");

  let _ = fs::remove_dir_all(&root);
  let _ = fs::remove_dir_all(&stray);

  for directory in [root.join("meshes"), root.join("textures"), level.clone(), stray.clone()] {
    fs::create_dir_all(directory).expect("directory created");
  }

  let directory = |path: &PathBuf| LevelSource::Directory {
    path: path.to_string_lossy().into_owned(),
  };

  assert_eq!(
    to_level_logical_path(&directory(&level), "level").unwrap().as_str(),
    "levels\\l01\\level"
  );
  assert!(
    to_level_logical_path(&directory(&stray), "level").is_err(),
    "a directory under no root is refused rather than read unrecorded"
  );
}
