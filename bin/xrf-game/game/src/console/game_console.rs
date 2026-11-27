use std::cell::RefCell;
use std::rc::Rc;

use xrf_game_console::{ConsoleAnswer, ConsoleCompletion, ConsoleRegistry};
use xrf_game_settings::GameSettingsStore;

use crate::console::game_console_commands::GameConsoleCommands;
use crate::console::game_console_effect::GameConsoleEffect;
use crate::console::game_console_state::GameConsoleState;

/// The game's console: its commands, and the state they run against, whose variables are the settings store's. It is
/// a client of the store, never its owner: a setting it sets is set there, and the shell applies it as it applies any
/// client's. The developer layer's drawer and the bridge enter lines alike; what a line asks of the running game
/// beyond the settings is taken as effects.
///
/// Its commands are registered in its registry (`ConsoleRegistry::register`), the hook point where a script will add a
/// command or override one; the game's own are [`GameConsoleCommands`].
pub(crate) struct GameConsole {
  registry: ConsoleRegistry<GameConsoleState>,
  state: GameConsoleState,
}

impl GameConsole {
  /// The console over the settings in `settings`, with the game's own commands.
  pub(crate) fn new(settings: Rc<RefCell<GameSettingsStore>>) -> Self {
    let mut registry: ConsoleRegistry<GameConsoleState> = ConsoleRegistry::default();

    for command in GameConsoleCommands::create() {
      registry.register(command);
    }

    Self {
      registry,
      state: GameConsoleState {
        settings,
        effects: Vec::new(),
      },
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
}
