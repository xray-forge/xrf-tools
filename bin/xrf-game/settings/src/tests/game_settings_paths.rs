use crate::game_preset::GamePreset;
use crate::game_setting_entry::GameSettingEntry;
use crate::game_settings::GameSettings;

#[test]
fn lists_every_setting_by_its_snake_case_path() {
  let entries: Vec<GameSettingEntry> = GamePreset::High.get_settings().get_entries();

  assert!(entries.windows(2).all(|pair| pair[0].path < pair[1].path));
  assert!(entries.iter().all(|entry| !entry.path.contains(char::is_uppercase)));
  assert!(entries.iter().any(|entry| entry.path == "view.shadows.resolution"));
  assert!(entries.iter().any(|entry| entry.path == "window.mode"));
}

#[test]
fn takes_back_every_value_it_lists_unchanged() {
  let settings: GameSettings = GamePreset::High.get_settings();

  for entry in settings.get_entries() {
    let set: Result<GameSettings, String> = settings.with_setting(&entry.path, &entry.value);

    assert_eq!(set.as_ref(), Ok(&settings), "{} = {}", entry.path, entry.value);
  }
}

#[test]
fn holds_a_value_set_to_the_renderers_bounds() {
  let settings: GameSettings = GameSettings::default()
    .with_setting("render.texture_lod", "9")
    .expect("a texture quality");

  assert_eq!(settings.render.texture_lod, 4);
}

#[test]
fn sets_one_setting_and_refuses_what_it_does_not_take() {
  let settings: GameSettings = GameSettings::default();
  let set: GameSettings = settings
    .with_setting("view.shadows.resolution", "1024")
    .expect("a resolution");

  assert_eq!(set.view.shadows.resolution, 1024);
  assert!(settings.with_setting("view.shadows.nothing", "1").is_err());
  assert!(settings.with_setting("view.shadows.resolution", "high").is_err());
  assert!(settings.with_setting("view.shadows", "1").is_err());
}
