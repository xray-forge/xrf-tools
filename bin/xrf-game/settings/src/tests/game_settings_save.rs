use std::fs;
use std::path::PathBuf;

use xrf_ltx::Ltx;

use crate::game_preset::GamePreset;
use crate::game_setting_entry::GameSettingEntry;
use crate::game_settings::GameSettings;

/// A settings file in a folder of its own under the system's temporary one, missing.
fn create_file(name: &str) -> PathBuf {
  let directory: PathBuf = std::env::temp_dir().join(format!("xrf-game-settings-{name}-{}", std::process::id()));

  let _ = fs::remove_dir_all(&directory);

  directory.join("settings.ltx")
}

fn change(path: &str, value: &str) -> GameSettingEntry {
  GameSettingEntry {
    path: String::from(path),
    value: String::from(value),
  }
}

#[test]
fn writes_the_preset_and_only_what_differs_from_it() {
  let preset: GameSettings = GamePreset::High.get_settings();
  let changed: GameSettings = preset
    .with_setting("view.shadows.resolution", "2048")
    .and_then(|settings| settings.with_setting("view.grass.is_shadowed", "false"))
    .expect("two changes");
  let ltx: Ltx = changed.create_settings_ltx(GamePreset::High);

  assert_eq!(
    ltx.sections().collect::<Vec<&str>>(),
    vec!["settings", "view.grass", "view.shadows"]
  );
  assert_eq!(ltx.get_from("settings", "preset"), Some("high"));
  assert_eq!(ltx.get_from("view.shadows", "resolution"), Some("2048"));
  assert_eq!(ltx.get_from("view.grass", "is_shadowed"), Some("false"));
  assert_eq!(preset.create_settings_ltx(GamePreset::High).sections().count(), 1);
}

#[test]
fn saves_changes_over_the_files_own_settings_and_reads_them_back() {
  let file: PathBuf = create_file("over");

  // A first save makes the file.
  GameSettings::save_changes(
    std::slice::from_ref(&file),
    &[change("view.shadows.resolution", "2048")],
  )
  .expect("saved");
  // A run's own arguments are not saved: only the changes given are.
  let run: GameSettings = GameSettings::load(
    std::slice::from_ref(&file),
    None,
    &[String::from("view.grass.radius=100")],
  );

  assert_eq!(run.view.shadows.resolution, 2048);
  assert!((run.view.grass.radius - 100.0).abs() < 1e-6);

  let written: PathBuf = GameSettings::save_changes(
    std::slice::from_ref(&file),
    &[change("view.grass.is_shadowed", "false")],
  )
  .expect("saved");
  let read: GameSettings = GameSettings::load(std::slice::from_ref(&file), None, &[]);
  let expected: GameSettings = GamePreset::High
    .get_settings()
    .with_setting("view.shadows.resolution", "2048")
    .and_then(|settings| settings.with_setting("view.grass.is_shadowed", "false"))
    .expect("two changes");

  assert_eq!(written, file);
  assert_eq!(read, expected);

  let _ = fs::remove_dir_all(file.parent().expect("a folder"));
}
