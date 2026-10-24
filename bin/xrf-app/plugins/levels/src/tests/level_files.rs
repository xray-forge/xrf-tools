use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::state::LevelSource;

fn zaton() -> LevelSource {
  LevelSource::Asset {
    logical_path: String::from("levels\\zaton"),
  }
}

#[test]
fn names_a_level_file_by_its_logical_path_inside_the_level() {
  assert_eq!(
    zaton().to_file_logical_path("level.geom").unwrap().as_str(),
    "levels\\zaton\\level.geom"
  );
  assert_eq!(
    zaton().to_file_logical_path("motions/door.anm").unwrap().as_str(),
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
    assert!(zaton().to_file_logical_path(file).is_err(), "'{file}' was taken");
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
    directory(&level).to_file_logical_path("level").unwrap().as_str(),
    "levels\\l01\\level"
  );
  assert!(
    directory(&stray).to_file_logical_path("level").is_err(),
    "a directory under no root is refused rather than read unrecorded"
  );
}
