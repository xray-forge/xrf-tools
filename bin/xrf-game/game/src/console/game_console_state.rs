use std::collections::BTreeMap;
use std::path::PathBuf;

use xrf_game_console::{ConsoleAnswer, ConsoleContext, ConsoleVariable};
use xrf_game_settings::{GameSettingEntry, GameSettings};

use crate::console::game_console_effect::GameConsoleEffect;

/// What the game's console runs against: the settings as they stand, its variables by their dotted paths, the changes
/// made through it not yet saved, and what the lines run since play last took them ask of the game.
pub(crate) struct GameConsoleState {
  pub(crate) settings: GameSettings,
  /// Where the settings file may be, best first, as loading reads it.
  pub(crate) files: Vec<PathBuf>,
  /// Each setting set through the console since the last save, by its path, at the value it took.
  pub(crate) unsaved: BTreeMap<String, String>,
  pub(crate) effects: Vec<GameConsoleEffect>,
}

impl GameConsoleState {
  /// The settings group the window opens by, which it reads only as it opens.
  const WINDOW_GROUP: &'static str = "window.";

  /// Saves the settings changed through the console into the settings file, answering where; nothing to save is said
  /// so.
  pub(crate) fn save(&mut self) -> ConsoleAnswer {
    if self.unsaved.is_empty() {
      return ConsoleAnswer::succeed(vec![String::from("No setting changed since the last save")]);
    }

    let changes: Vec<GameSettingEntry> = self
      .unsaved
      .iter()
      .map(|(path, value)| GameSettingEntry {
        path: path.clone(),
        value: value.clone(),
      })
      .collect();

    match GameSettings::save_changes(&self.files, &changes) {
      Ok(file) => {
        self.unsaved.clear();

        ConsoleAnswer::succeed(vec![format!(
          "Saved {} {} to '{}'",
          changes.len(),
          if changes.len() == 1 { "setting" } else { "settings" },
          file.display()
        )])
      }
      Err(error) => ConsoleAnswer::fail(format!("The settings were not saved: {error}")),
    }
  }
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

  /// Sets the setting at the path `name`, answering it as it now reads, held to the renderer's bounds, and keeps it to
  /// save.
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
    self.unsaved.insert(name.to_owned(), set);
    self.effects.push(GameConsoleEffect::ApplySettings);

    ConsoleAnswer::succeed(lines)
  }
}
