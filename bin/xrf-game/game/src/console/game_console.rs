use xrf_game_console::{ConsoleAnswer, ConsoleCommand, ConsoleRegistry};
use xrf_game_movement::MovementCommand;
use xrf_game_settings::GameSettings;

use crate::console::game_console_effect::GameConsoleEffect;
use crate::console::game_console_state::GameConsoleState;

/// The game's console: its commands, and the state they run against, the settings its variables are. The developer
/// layer's drawer and the bridge enter lines alike; what a line asks of the running game is taken as effects.
pub(crate) struct GameConsole {
  registry: ConsoleRegistry<GameConsoleState>,
  state: GameConsoleState,
}

impl GameConsole {
  /// The console over `settings` as the game started with them.
  pub(crate) fn new(settings: GameSettings) -> Self {
    let registry: ConsoleRegistry<GameConsoleState> = ConsoleRegistry::default()
      .with(ConsoleCommand {
        name: "quit",
        usage: "",
        help: "quits the game",
        run: |state: &mut GameConsoleState, _: Option<&str>| {
          state.effects.push(GameConsoleEffect::Quit);

          ConsoleAnswer::succeed(vec![String::from("Quitting")])
        },
      })
      .with(ConsoleCommand {
        name: "noclip",
        usage: "",
        help: "switches between walking and flying through everything",
        run: |state: &mut GameConsoleState, _: Option<&str>| {
          state
            .effects
            .push(GameConsoleEffect::Movement(MovementCommand::ToggleMode));

          ConsoleAnswer::succeed(vec![String::from("Movement switched")])
        },
      });

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

  /// What the lines run since the last take ask of the game, in order.
  pub(crate) fn take_effects(&mut self) -> Vec<GameConsoleEffect> {
    std::mem::take(&mut self.state.effects)
  }

  /// The settings as the console's lines left them.
  pub(crate) fn get_settings(&self) -> &GameSettings {
    &self.state.settings
  }
}
