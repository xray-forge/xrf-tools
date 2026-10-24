use std::fs;
use std::path::{Path, PathBuf};

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::viewer::LevelViewerFolder;

#[test]
fn numbers_a_screenshot_whose_name_is_taken() {
  let folder: PathBuf = build_absolute_generated_test_resource_path("level-viewer-free-screenshot");

  fs::create_dir_all(&folder).expect("the folder to be made");

  for name in ["zaton.png", "zaton-2.png", "zaton-3.png"] {
    let _ = fs::remove_file(folder.join(name));
  }

  assert_eq!(
    LevelViewerFolder::find_free_screenshot(&folder, "zaton").expect("a free name"),
    folder.join("zaton.png")
  );

  fs::write(folder.join("zaton.png"), b"").expect("the first to be written");
  fs::write(folder.join("zaton-2.png"), b"").expect("the second to be written");

  assert_eq!(
    LevelViewerFolder::find_free_screenshot(&folder, "zaton").expect("a free name"),
    folder.join("zaton-3.png")
  );
}

#[test]
fn refuses_a_screenshot_name_that_is_not_one_file_name() {
  for name in [
    "",
    "..",
    ".hidden",
    "nested/zaton",
    "..\\zaton",
    "C:zaton",
    "zaton?",
    "za|ton",
  ] {
    assert!(
      LevelViewerFolder::find_free_screenshot(Path::new("screenshots"), name).is_err(),
      "'{name}' must not name a screenshot"
    );
  }
}

#[test]
fn takes_only_a_json_file_for_a_view() {
  assert!(LevelViewerFolder::require_view_file(Path::new("C:/views/zaton.json")).is_ok());
  assert!(LevelViewerFolder::require_view_file(Path::new("C:/views/zaton.JSON")).is_ok());
  assert!(LevelViewerFolder::require_view_file(Path::new("C:/views/zaton.png")).is_err());
  assert!(LevelViewerFolder::require_view_file(Path::new("C:/Windows/win.ini")).is_err());
  assert!(LevelViewerFolder::require_view_file(Path::new("C:/views/zaton")).is_err());
}

#[test]
fn takes_only_an_image_directly_in_the_screenshots_folder() {
  let folder: &Path = Path::new("appdata/level-viewer/screenshots");

  assert!(LevelViewerFolder::is_screenshot(
    folder,
    &folder.join("zaton-20261020-070509.png")
  ));
  assert!(LevelViewerFolder::is_screenshot(folder, &folder.join("zaton.PNG")));
  assert!(!LevelViewerFolder::is_screenshot(folder, &folder.join("zaton.json")));
  assert!(!LevelViewerFolder::is_screenshot(
    folder,
    &folder.join("nested").join("zaton.png")
  ));
  assert!(!LevelViewerFolder::is_screenshot(
    folder,
    &folder.join("..").join("logs.png")
  ));
  assert!(!LevelViewerFolder::is_screenshot(
    folder,
    Path::new("C:/Windows/zaton.png")
  ));
}
