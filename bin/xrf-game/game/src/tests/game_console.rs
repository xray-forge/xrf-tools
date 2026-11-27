use std::fs;
use std::path::PathBuf;

use xrf_game_console::ConsoleAnswer;
use xrf_game_movement::MovementCommand;
use xrf_game_settings::{GamePreset, GameSettings};

use crate::console::game_console::GameConsole;
use crate::console::game_console_effect::GameConsoleEffect;

#[test]
fn sets_a_setting_by_its_path_and_asks_the_view_to_draw_with_it() {
  let mut console: GameConsole = GameConsole::new(GamePreset::High.get_settings(), Vec::new());
  let answer: ConsoleAnswer = console.execute("view.shadows.resolution 1024");

  assert_eq!(
    answer,
    ConsoleAnswer::succeed(vec![String::from("view.shadows.resolution = 1024")])
  );
  assert_eq!(console.get_settings().view.shadows.resolution, 1024);
  assert_eq!(console.take_effects(), vec![GameConsoleEffect::ApplySettings]);
  assert_eq!(
    console.execute("view.shadows.resolution").lines,
    vec!["view.shadows.resolution = 1024"]
  );
}

#[test]
fn leaves_a_setting_as_it_is_for_a_value_it_does_not_take() {
  let settings: GameSettings = GamePreset::High.get_settings();
  let mut console: GameConsole = GameConsole::new(settings.clone(), Vec::new());

  assert!(console.execute("view.shadows.resolution high").is_failed);
  assert_eq!(console.get_settings(), &settings);
  assert!(console.take_effects().is_empty());
}

#[test]
fn says_a_window_setting_waits_for_the_next_start() {
  let mut console: GameConsole = GameConsole::new(GameSettings::default(), Vec::new());

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
  let mut console: GameConsole = GameConsole::new(GamePreset::High.get_settings(), vec![file.clone()]);

  assert_eq!(
    console.execute("save").lines,
    vec!["No setting changed since the last save"]
  );

  console.execute("view.shadows.resolution 1024");

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
fn switches_movement_and_quits_as_effects() {
  let mut console: GameConsole = GameConsole::new(GameSettings::default(), Vec::new());

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
