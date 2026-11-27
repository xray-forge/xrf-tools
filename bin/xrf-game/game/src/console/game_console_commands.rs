use xrf_game_console::{ConsoleAnswer, ConsoleCommand};
use xrf_game_movement::MovementCommand;

use crate::console::game_console_effect::GameConsoleEffect;
use crate::console::game_console_state::GameConsoleState;
use crate::state::game_command::GameCommand;

/// The game's own console commands. A command that changes a system sends that system's command through the game loop
/// as an effect, so the console knows no system, and a system's command is added here beside the rest; a script's,
/// registered later in the console's registry, joins or overrides them.
pub(crate) struct GameConsoleCommands;

impl GameConsoleCommands {
  /// Every command the game starts its console with.
  pub(crate) fn create() -> Vec<ConsoleCommand<GameConsoleState>> {
    vec![
      ConsoleCommand::new("quit", "", "quits the game", |state: &mut GameConsoleState, _| {
        state.effects.push(GameConsoleEffect::Quit);

        ConsoleAnswer::succeed(vec![String::from("Quitting")])
      }),
      ConsoleCommand::new(
        "save",
        "",
        "saves the settings changed here to the settings file",
        |state: &mut GameConsoleState, _| state.save(),
      ),
      ConsoleCommand::new(
        "noclip",
        "",
        "switches between walking and flying through everything",
        |state: &mut GameConsoleState, _| {
          state.effects.push(GameConsoleEffect::Command(GameCommand::Movement(
            MovementCommand::ToggleMode,
          )));

          ConsoleAnswer::succeed(vec![String::from("Movement switched")])
        },
      ),
    ]
  }
}
