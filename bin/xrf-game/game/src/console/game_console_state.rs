use xrf_game_console::{ConsoleAnswer, ConsoleContext, ConsoleVariable};
use xrf_game_settings::{GameSettingEntry, GameSettings};

use crate::console::game_console_effect::GameConsoleEffect;

/// What the game's console runs against: the settings as they stand, its variables by their dotted paths, and what the
/// lines run since play last took them ask of the game.
pub(crate) struct GameConsoleState {
  pub(crate) settings: GameSettings,
  pub(crate) effects: Vec<GameConsoleEffect>,
}

impl GameConsoleState {
  /// The settings group the window opens by, which it reads only as it opens.
  const WINDOW_GROUP: &'static str = "window.";
}

impl ConsoleContext for GameConsoleState {
  fn list_variables(&self) -> Vec<ConsoleVariable> {
    self
      .settings
      .get_entries()
      .into_iter()
      .map(|GameSettingEntry { path, value }| ConsoleVariable { name: path, value })
      .collect()
  }

  /// Sets the setting at the path `name`, answering it as it now reads; the renderer holds a number to its bounds as it
  /// takes it.
  fn set_variable(&mut self, name: &str, value: &str) -> ConsoleAnswer {
    let settings: GameSettings = match self.settings.with_setting(name, value) {
      Ok(settings) => settings,
      Err(reason) => return ConsoleAnswer::fail(format!("{name} left as it is: {reason}")),
    };
    let set: String = settings
      .get_entries()
      .into_iter()
      .find(|entry| entry.path == name)
      .map_or_else(|| String::from(value), |entry| entry.value);
    let mut lines: Vec<String> = vec![format!("{name} = {set}")];

    if name.starts_with(Self::WINDOW_GROUP) {
      lines.push(String::from("The window takes it the next time the game starts"));
    }

    self.settings = settings;
    self.effects.push(GameConsoleEffect::ApplySettings);

    ConsoleAnswer::succeed(lines)
  }
}
