use xrf_game_console::ConsoleAnswer;
use xrf_game_movement::MovementCommand;
use xrf_game_settings::{GamePreset, GameSettings};

use crate::console::game_console::GameConsole;
use crate::console::game_console_effect::GameConsoleEffect;

#[test]
fn sets_a_setting_by_its_path_and_asks_the_view_to_draw_with_it() {
  let mut console: GameConsole = GameConsole::new(GamePreset::High.get_settings());
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
  let mut console: GameConsole = GameConsole::new(settings.clone());

  assert!(console.execute("view.shadows.resolution high").is_failed);
  assert_eq!(console.get_settings(), &settings);
  assert!(console.take_effects().is_empty());
}

#[test]
fn says_a_window_setting_waits_for_the_next_start() {
  let mut console: GameConsole = GameConsole::new(GameSettings::default());

  assert_eq!(
    console.execute("window.width 1600").lines,
    vec![
      "window.width = 1600",
      "The window takes it the next time the game starts"
    ]
  );
}

#[test]
fn switches_movement_and_quits_as_effects() {
  let mut console: GameConsole = GameConsole::new(GameSettings::default());

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
