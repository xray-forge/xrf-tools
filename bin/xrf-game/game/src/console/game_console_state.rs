use std::cell::{RefCell, RefMut};
use std::path::Path;
use std::rc::Rc;

use xrf_game_console::{ConsoleAnswer, ConsoleContext, ConsoleVariable};
use xrf_game_settings::{GameSettingEntry, GameSettingsStore};

use crate::console::game_console_effect::GameConsoleEffect;
use crate::input::game_action::GameAction;
use crate::input::game_bindings::GameBindings;
use crate::input::game_key_names::GameKeyNames;

/// What the game's console runs against: the settings store, a client of it as the options menu will be, whose
/// settings are the console's variables, and what the lines run since play last took them ask of the game.
pub(crate) struct GameConsoleState {
  pub(crate) settings: Rc<RefCell<GameSettingsStore>>,
  pub(crate) effects: Vec<GameConsoleEffect>,
}

impl GameConsoleState {
  /// The settings group the window opens by, which it reads only as it opens.
  const WINDOW_GROUP: &'static str = "window.";

  /// The settings groups of the keys bound in place of the tree's, main and second.
  pub(crate) const BINDINGS: &'static str = "input.bindings";
  pub(crate) const BINDINGS_SECONDARY: &'static str = "input.bindings_secondary";

  /// The setting `sensitivity` sets.
  pub(crate) const SENSITIVITY: &'static str = "input.sensitivity";

  /// Binds the action `argument` names to the key it names next, in the bindings group `group`, by SDL's name (an
  /// X-Ray name read as SDL's), or to none, restoring the tree's; shows the binding where it names no key.
  pub(crate) fn bind(&mut self, group: &str, argument: Option<&str>) -> ConsoleAnswer {
    let line: &str = argument.unwrap_or_default().trim();
    let (name, key) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let Some(action) = GameAction::of_name(name) else {
      let names: Vec<&str> = GameAction::ALL.into_iter().map(GameAction::get_name).collect();

      return ConsoleAnswer::fail(format!("No action is named '{name}': bind one of {}", names.join(", ")));
    };
    let path: String = format!("{group}.{}", action.get_name());
    let key: &str = key.trim();

    if key.is_empty() || key.eq_ignore_ascii_case("none") {
      return self.set_or_show(&path, (!key.is_empty()).then_some(key));
    }

    if GameBindings::find_key(key).is_none() {
      return ConsoleAnswer::fail(format!(
        "No key is named '{key}': name it as SDL does, as W or Left Shift"
      ));
    }

    let sdl: String = GameKeyNames::to_sdl(key).unwrap_or_else(|| String::from(key));

    self.set_or_show(&path, Some(&sdl))
  }

  /// Sets the setting at `path` to `value`, or shows it where there is none.
  pub(crate) fn set_or_show(&mut self, path: &str, value: Option<&str>) -> ConsoleAnswer {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
      Some(value) => self.set_variable(path, value),
      None => {
        let shown: Option<String> = self
          .settings
          .borrow()
          .get_entries()
          .into_iter()
          .find(|entry| entry.path == path)
          .map(|entry| format!("{} = {}", entry.path, entry.value));

        shown.map_or_else(
          || ConsoleAnswer::fail(format!("The settings hold no {path}")),
          |line| ConsoleAnswer::succeed(vec![line]),
        )
      }
    }
  }

  /// Saves the settings changed since the last save into the settings file, answering where; nothing to save is said
  /// so.
  pub(crate) fn save(&mut self) -> ConsoleAnswer {
    let mut settings: RefMut<'_, GameSettingsStore> = self.settings.borrow_mut();
    let count: usize = settings.count_unsaved();

    match settings.save() {
      Ok(None) => ConsoleAnswer::succeed(vec![String::from("No setting changed since the last save")]),
      Ok(Some(file)) => ConsoleAnswer::succeed(vec![Self::describe_saved(count, &file)]),
      Err(error) => ConsoleAnswer::fail(format!("The settings were not saved: {error}")),
    }
  }

  /// How a save of `count` settings to `file` is said.
  fn describe_saved(count: usize, file: &Path) -> String {
    format!(
      "Saved {count} {} to '{}'",
      if count == 1 { "setting" } else { "settings" },
      file.display()
    )
  }
}

impl ConsoleContext for GameConsoleState {
  fn list_variables(&self) -> Vec<ConsoleVariable> {
    self
      .settings
      .borrow()
      .get_entries()
      .into_iter()
      .map(|GameSettingEntry { path, value }| ConsoleVariable { name: path, value })
      .collect()
  }

  /// Sets the setting at the path `name` in the store, answering it as it now reads, held to the renderer's bounds.
  fn set_variable(&mut self, name: &str, value: &str) -> ConsoleAnswer {
    let entry: GameSettingEntry = match self.settings.borrow_mut().set(name, value) {
      Ok(entry) => entry,
      Err(reason) => return ConsoleAnswer::fail(format!("{name} left as it is: {reason}")),
    };
    let mut lines: Vec<String> = vec![format!("{} = {}", entry.path, entry.value)];

    if name.starts_with(Self::WINDOW_GROUP) {
      lines.push(String::from("The window takes it the next time the game starts"));
    }

    ConsoleAnswer::succeed(lines)
  }
}
