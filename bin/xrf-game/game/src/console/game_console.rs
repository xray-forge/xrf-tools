use xrf_game_console::{ConsoleAnswer, ConsoleCompletion, ConsoleRegistry};
use xrf_game_settings::GameSettingsStore;

use crate::console::game_console_commands::GameConsoleCommands;
use crate::console::game_console_effect::GameConsoleEffect;
use crate::console::game_console_family::GameConsoleFamily;
use crate::console::game_console_state::GameConsoleState;

/// The game's console: its commands, and what the lines run since play last took them ask of the game. Its variables
/// are the settings store's, which play owns and lends it for each line (`execute(&mut store, line)`); a setting a line
/// sets is set there, and play applies it as it applies any change. The developer layer's drawer and the bridge enter
/// lines alike.
///
/// Its commands are registered in its registry (`ConsoleRegistry::register`), the hook point where a script will add a
/// command or override one; the game's own are [`GameConsoleCommands`].
pub(crate) struct GameConsole {
  registry: ConsoleRegistry<GameConsoleFamily>,
  effects: Vec<GameConsoleEffect>,
}

impl Default for GameConsole {
  /// The console with the game's own commands.
  fn default() -> Self {
    let mut registry: ConsoleRegistry<GameConsoleFamily> = ConsoleRegistry::default();

    for command in GameConsoleCommands::create() {
      registry.register(command);
    }

    Self {
      registry,
      effects: Vec::new(),
    }
  }
}

impl GameConsole {
  /// Runs the line `text` against `settings`.
  pub(crate) fn execute(&mut self, settings: &mut GameSettingsStore, text: &str) -> ConsoleAnswer {
    self.registry.execute(
      &mut GameConsoleState {
        settings,
        effects: &mut self.effects,
      },
      text,
    )
  }

  /// Completes the name `line` starts, as Tab does, `settings`' paths among the names.
  pub(crate) fn complete(&mut self, settings: &mut GameSettingsStore, line: &str) -> ConsoleCompletion {
    self.registry.complete(
      &GameConsoleState {
        settings,
        effects: &mut self.effects,
      },
      line,
    )
  }

  /// What the lines run since the last take ask of the game, in order.
  pub(crate) fn take_effects(&mut self) -> Vec<GameConsoleEffect> {
    std::mem::take(&mut self.effects)
  }
}
