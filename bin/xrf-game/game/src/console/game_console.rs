use std::collections::BTreeMap;
use std::path::PathBuf;

use xrf_game_console::{ConsoleAnswer, ConsoleCompletion, ConsoleRegistry};
use xrf_game_settings::GameSettings;

use crate::console::game_console_commands::GameConsoleCommands;
use crate::console::game_console_effect::GameConsoleEffect;
use crate::console::game_console_state::GameConsoleState;

/// The game's console: its commands, and the state they run against, the settings its variables are. The developer
/// layer's drawer and the bridge enter lines alike; what a line asks of the running game is taken as effects. Settings
/// changed through it are saved by `save`, and as the game quits, as X-Ray saves its console's on exit.
///
/// Its commands are registered in its registry (`ConsoleRegistry::register`), the hook point where a script will add a
/// command or override one; the game's own are [`GameConsoleCommands`].
pub(crate) struct GameConsole {
  registry: ConsoleRegistry<GameConsoleState>,
  state: GameConsoleState,
}

impl GameConsole {
  /// The console over `settings` as the game started with them, with the game's own commands, saving its changes to
  /// the first of `files` that exists, else the first.
  pub(crate) fn new(settings: GameSettings, files: Vec<PathBuf>) -> Self {
    let mut registry: ConsoleRegistry<GameConsoleState> = ConsoleRegistry::default();

    for command in GameConsoleCommands::create() {
      registry.register(command);
    }

    Self {
      registry,
      state: GameConsoleState {
        settings,
        files,
        unsaved: BTreeMap::new(),
        effects: Vec::new(),
      },
    }
  }

  /// Saves the settings changed through the console and not saved yet, as the game quits, logging what it did; nothing
  /// where none changed.
  pub(crate) fn save_on_quit(&mut self) {
    if self.state.unsaved.is_empty() {
      return;
    }

    let answer: ConsoleAnswer = self.state.save();

    for line in &answer.lines {
      if answer.is_failed {
        log::error!("{line}");
      } else {
        log::info!("{line}");
      }
    }
  }

  /// Runs the line `text`.
  pub(crate) fn execute(&mut self, text: &str) -> ConsoleAnswer {
    self.registry.execute(&mut self.state, text)
  }

  /// Completes the name `line` starts, as Tab does.
  pub(crate) fn complete(&self, line: &str) -> ConsoleCompletion {
    self.registry.complete(&self.state, line)
  }

  /// What the lines run since the last take ask of the game, in order.
  pub(crate) fn take_effects(&mut self) -> Vec<GameConsoleEffect> {
    std::mem::take(&mut self.state.effects)
  }

  /// The settings as the console's lines left them.
  pub(crate) fn get_settings(&self) -> &GameSettings {
    &self.state.settings
  }
}
