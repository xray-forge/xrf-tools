use crate::game_preset::GamePreset;
use crate::game_settings_changes::GameSettingsChanges;
use crate::game_settings_group::GameSettingsGroup;
use crate::game_settings_store::GameSettingsStore;

#[test]
fn sets_a_setting_and_keeps_its_group_as_changed_until_taken() {
  let mut store: GameSettingsStore = GameSettingsStore::new(GamePreset::High.get_settings(), Vec::new());

  assert_eq!(
    store.set("view.shadows.resolution", "1024").map(|entry| entry.value),
    Ok(String::from("1024"))
  );
  assert_eq!(
    store.set("window.width", "1600").map(|entry| entry.value),
    Ok(String::from("1600"))
  );
  assert_eq!(store.get().view.shadows.resolution, 1024);
  assert_eq!(store.count_unsaved(), 2);

  let changes: GameSettingsChanges = store.take_changes();

  assert!(changes.has(GameSettingsGroup::View) && changes.has(GameSettingsGroup::Window));
  assert!(!changes.has_any(&[GameSettingsGroup::Render, GameSettingsGroup::Upscaling]));
  assert!(store.take_changes().is_empty());
}

#[test]
fn leaves_the_settings_and_their_changes_as_they_are_for_a_value_a_setting_does_not_take() {
  let mut store: GameSettingsStore = GameSettingsStore::new(GamePreset::High.get_settings(), Vec::new());

  assert!(store.set("view.shadows.resolution", "high").is_err());
  assert!(store.set("view.nothing", "1").is_err());
  assert_eq!(store.get(), &GamePreset::High.get_settings());
  assert!(store.take_changes().is_empty());
  assert_eq!(store.count_unsaved(), 0);
}

#[test]
fn saves_nothing_where_nothing_changed() {
  let mut store: GameSettingsStore = GameSettingsStore::new(GamePreset::High.get_settings(), Vec::new());

  assert_eq!(store.save().ok(), Some(None));
}

#[test]
fn groups_a_path_by_its_first_part() {
  assert_eq!(
    GameSettingsGroup::of_path("render.backend"),
    Some(GameSettingsGroup::Render)
  );
  assert_eq!(
    GameSettingsGroup::of_path("upscaling.mode"),
    Some(GameSettingsGroup::Upscaling)
  );
  assert_eq!(GameSettingsGroup::of_path("sound.volume"), None);
}
