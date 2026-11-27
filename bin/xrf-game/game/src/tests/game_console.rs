use std::fs;
use std::path::PathBuf;

use xrf_game_console::ConsoleAnswer;
use xrf_game_movement::MovementCommand;
use xrf_game_settings::{GamePreset, GameSettings, GameSettingsGroup, GameSettingsStore};

use crate::console::game_console::GameConsole;
use crate::console::game_console_effect::GameConsoleEffect;

/// The console and the store it is lent for each line, as play holds them.
struct Played {
  store: GameSettingsStore,
  console: GameConsole,
}

impl Played {
  /// A store over `settings` saving to `files`, and the game's console.
  fn new(settings: GameSettings, files: Vec<PathBuf>) -> Self {
    Self {
      store: GameSettingsStore::new(settings, files),
      console: GameConsole::default(),
    }
  }

  /// Runs `line` with the store lent to the console.
  fn run(&mut self, line: &str) -> ConsoleAnswer {
    self.console.execute(&mut self.store, line)
  }
}

#[test]
fn sets_a_setting_in_the_store_which_tells_its_group_changed() {
  let mut played: Played = Played::new(GamePreset::High.get_settings(), Vec::new());
  let answer: ConsoleAnswer = played.run("view.shadows.resolution 1024");

  assert_eq!(
    answer,
    ConsoleAnswer::succeed(vec![String::from("view.shadows.resolution = 1024")])
  );
  assert_eq!(played.store.get().view.shadows.resolution, 1024);
  assert!(played.store.take_changes().has(GameSettingsGroup::View));
  assert!(played.store.take_changes().is_empty());
  assert_eq!(
    played.run("view.shadows.resolution").lines,
    vec!["view.shadows.resolution = 1024"]
  );
}

#[test]
fn leaves_a_setting_as_it_is_for_a_value_it_does_not_take() {
  let settings: GameSettings = GamePreset::High.get_settings();
  let mut played: Played = Played::new(settings.clone(), Vec::new());

  assert!(played.run("view.shadows.resolution high").is_failed);
  assert_eq!(played.store.get(), &settings);
  assert!(played.store.take_changes().is_empty());
}

#[test]
fn says_a_window_setting_waits_for_the_next_start() {
  let mut played: Played = Played::new(GameSettings::default(), Vec::new());

  assert_eq!(
    played.run("window.width 1600").lines,
    vec![
      "window.width = 1600",
      "The window takes it the next time the game starts"
    ]
  );
}

#[test]
fn completes_a_name_from_the_settings_lent_to_it() {
  let mut played: Played = Played::new(GameSettings::default(), Vec::new());
  let line: String = played.console.complete(&mut played.store, "input.sens").line;

  assert_eq!(line, "input.sensitivity ");
}

#[test]
fn saves_the_settings_changed_here_once_and_says_so() {
  let directory: PathBuf = std::env::temp_dir().join(format!("xrf-game-console-save-{}", std::process::id()));
  let file: PathBuf = directory.join("settings.ltx");
  let _ = fs::remove_dir_all(&directory);
  let mut played: Played = Played::new(GamePreset::High.get_settings(), vec![file.clone()]);

  assert_eq!(played.run("save").lines, vec!["No setting changed since the last save"]);

  played.run("view.shadows.resolution 1024");

  assert_eq!(played.store.count_unsaved(), 1);
  assert_eq!(
    played.run("save").lines,
    vec![format!("Saved 1 setting to '{}'", file.display())]
  );
  assert_eq!(
    GameSettings::load(std::slice::from_ref(&file), None, &[])
      .view
      .shadows
      .resolution,
    1024
  );
  assert_eq!(played.run("save").lines, vec!["No setting changed since the last save"]);

  let _ = fs::remove_dir_all(&directory);
}

#[test]
fn binds_keys_and_sets_the_sensitivity_through_the_store() {
  let mut played: Played = Played::new(GameSettings::default(), Vec::new());

  assert_eq!(played.run("bind forward Up").lines, vec!["input.bindings.forward = Up"]);
  assert_eq!(
    played.run("bind_sec forward kW").lines,
    vec!["input.bindings_secondary.forward = W"]
  );
  assert_eq!(played.run("sensitivity 0.004").lines, vec!["input.sensitivity = 0.004"]);
  assert_eq!(played.run("sensitivity").lines, vec!["input.sensitivity = 0.004"]);
  assert!(played.store.take_changes().has(GameSettingsGroup::Input));
  assert_eq!(played.store.get().input.bindings.forward.as_deref(), Some("Up"));

  // None restores the tree's binding; unknown actions and keys are refused.
  assert_eq!(
    played.run("bind forward none").lines,
    vec!["input.bindings.forward = none"]
  );
  assert_eq!(played.store.get().input.bindings.forward, None);
  assert!(played.run("bind fly W").is_failed);
  assert!(played.run("bind forward Nowhere").is_failed);
}

#[test]
fn switches_movement_and_quits_as_effects() {
  let mut played: Played = Played::new(GameSettings::default(), Vec::new());

  played.run("noclip");
  played.run("quit");

  assert_eq!(
    played.console.take_effects(),
    vec![
      GameConsoleEffect::Movement(MovementCommand::ToggleMode),
      GameConsoleEffect::Quit
    ]
  );
}
