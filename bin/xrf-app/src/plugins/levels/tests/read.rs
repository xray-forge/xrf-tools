use std::path::{Path, PathBuf};

use crate::plugins::levels::read::to_level_file_path;

#[test]
fn finds_a_level_file_by_its_name_inside_the_directory() {
  let directory: PathBuf = Path::new("levels").join("zaton");

  assert_eq!(
    to_level_file_path(&directory, "level.geom"),
    Ok(directory.join("level.geom"))
  );
  assert_eq!(
    to_level_file_path(&directory, "motions/door.anm"),
    Ok(directory.join("motions/door.anm"))
  );
}

// A spawn names an object's motion by a path its author wrote, which must not reach outside the level.
#[test]
fn refuses_a_name_leaving_the_directory() {
  let directory: PathBuf = Path::new("levels").join("zaton");

  for file in [
    "",
    "../level.geom",
    "motions/../../secret",
    "/etc/passwd",
    "./level.geom",
  ] {
    assert!(to_level_file_path(&directory, file).is_err(), "'{file}' was taken");
  }

  if cfg!(windows) {
    for file in ["..\\level.geom", "C:\\Windows\\win.ini", "\\level.geom"] {
      assert!(to_level_file_path(&directory, file).is_err(), "'{file}' was taken");
    }
  }
}
