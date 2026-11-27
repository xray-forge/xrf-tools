use std::path::PathBuf;

use crate::directories::game_directories::GameDirectories;

#[test]
fn keeps_its_own_files_under_its_folder_in_appdata_then_its_local_data() {
  assert_eq!(
    GameDirectories::order_own_directories(Some(PathBuf::from("game")), Some(PathBuf::from("data"))),
    vec![
      PathBuf::from("game").join("appdata").join("xrf-game"),
      PathBuf::from("data")
    ]
  );
  assert_eq!(
    GameDirectories::order_own_directories(None, Some(PathBuf::from("data"))),
    vec![PathBuf::from("data")]
  );
}

#[test]
fn reads_its_settings_from_its_own_directories() {
  let directories: GameDirectories = GameDirectories::new(Some(PathBuf::from("data")));

  assert_eq!(
    directories.get_settings_files(),
    directories
      .get_own_directories()
      .into_iter()
      .map(|directory| directory.join("settings.ltx"))
      .collect::<Vec<PathBuf>>()
  );
}
