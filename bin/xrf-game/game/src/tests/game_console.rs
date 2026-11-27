use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use xrf_game_console::ConsoleAnswer;
use xrf_game_movement::MovementCommand;
use xrf_game_settings::{GamePreset, GameSettings, GameSettingsGroup, GameSettingsStore};

use crate::console::game_console::GameConsole;
use crate::console::game_console_effect::GameConsoleEffect;

/// A store over `settings` saving to `files`, and the console over it.
fn console_over(settings: GameSettings, files: Vec<PathBuf>) -> (Rc<RefCell<GameSettingsStore>>, GameConsole) {
  let store: Rc<RefCell<GameSettingsStore>> = Rc::new(RefCell::new(GameSettingsStore::new(settings, files)));
  let console: GameConsole = GameConsole::new(Rc::clone(&store));

  (store, console)
}

#[test]
fn sets_a_setting_in_the_store_which_tells_its_group_changed() {
  let (store, mut console) = console_over(GamePreset::High.get_settings(), Vec::new());
  let answer: ConsoleAnswer = console.execute("view.shadows.resolution 1024");

  assert_eq!(
    answer,
    ConsoleAnswer::succeed(vec![String::from("view.shadows.resolution = 1024")])
  );
  assert_eq!(store.borrow().get().view.shadows.resolution, 1024);
  assert!(store.borrow_mut().take_changes().has(GameSettingsGroup::View));
  assert!(store.borrow_mut().take_changes().is_empty());
  assert_eq!(
    console.execute("view.shadows.resolution").lines,
    vec!["view.shadows.resolution = 1024"]
  );
}

#[test]
fn leaves_a_setting_as_it_is_for_a_value_it_does_not_take() {
  let settings: GameSettings = GamePreset::High.get_settings();
  let (store, mut console) = console_over(settings.clone(), Vec::new());

  assert!(console.execute("view.shadows.resolution high").is_failed);
  assert_eq!(store.borrow().get(), &settings);
  assert!(store.borrow_mut().take_changes().is_empty());
}

#[test]
fn says_a_window_setting_waits_for_the_next_start() {
  let (_, mut console) = console_over(GameSettings::default(), Vec::new());

  assert_eq!(
    console.execute("window.width 1600").lines,
    vec![
      "window.width = 1600",
      "The window takes it the next time the game starts"
    ]
  );
}

#[test]
fn saves_the_settings_changed_here_once_and_says_so() {
  let directory: PathBuf = std::env::temp_dir().join(format!("xrf-game-console-save-{}", std::process::id()));
  let file: PathBuf = directory.join("settings.ltx");
  let _ = fs::remove_dir_all(&directory);
  let (store, mut console) = console_over(GamePreset::High.get_settings(), vec![file.clone()]);

  assert_eq!(
    console.execute("save").lines,
    vec!["No setting changed since the last save"]
  );

  console.execute("view.shadows.resolution 1024");

  assert_eq!(store.borrow().count_unsaved(), 1);
  assert_eq!(
    console.execute("save").lines,
    vec![format!("Saved 1 setting to '{}'", file.display())]
  );
  assert_eq!(
    GameSettings::load(std::slice::from_ref(&file), None, &[])
      .view
      .shadows
      .resolution,
    1024
  );
  assert_eq!(
    console.execute("save").lines,
    vec!["No setting changed since the last save"]
  );

  let _ = fs::remove_dir_all(&directory);
}

#[test]
fn binds_keys_and_sets_the_sensitivity_through_the_store() {
  let (store, mut console) = console_over(GameSettings::default(), Vec::new());

  assert_eq!(
    console.execute("bind forward Up").lines,
    vec!["input.bindings.forward = Up"]
  );
  assert_eq!(
    console.execute("bind_sec forward kW").lines,
    vec!["input.bindings_secondary.forward = W"]
  );
  assert_eq!(
    console.execute("sensitivity 0.004").lines,
    vec!["input.sensitivity = 0.004"]
  );
  assert_eq!(console.execute("sensitivity").lines, vec!["input.sensitivity = 0.004"]);
  assert!(store.borrow_mut().take_changes().has(GameSettingsGroup::Input));
  assert_eq!(store.borrow().get().input.bindings.forward.as_deref(), Some("Up"));

  // None restores the tree's binding; unknown actions and keys are refused.
  assert_eq!(
    console.execute("bind forward none").lines,
    vec!["input.bindings.forward = none"]
  );
  assert_eq!(store.borrow().get().input.bindings.forward, None);
  assert!(console.execute("bind fly W").is_failed);
  assert!(console.execute("bind forward Nowhere").is_failed);
}

#[test]
fn switches_movement_and_quits_as_effects() {
  let (_, mut console) = console_over(GameSettings::default(), Vec::new());

  console.execute("noclip");
  console.execute("quit");

  assert_eq!(
    console.take_effects(),
    vec![
      GameConsoleEffect::Movement(MovementCommand::ToggleMode),
      GameConsoleEffect::Quit
    ]
  );
}
