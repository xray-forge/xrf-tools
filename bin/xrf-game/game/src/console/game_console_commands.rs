use xrf_game_console::{ConsoleAnswer, ConsoleCommand};
use xrf_game_movement::MovementCommand;

use crate::console::game_console_effect::GameConsoleEffect;
use crate::console::game_console_state::GameConsoleState;

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
        "bind",
        "<action> <key>",
        "binds an action to a key in place of the tree's (forward, back, lstrafe, rstrafe, jump, crouch, accel, \
         sprint_toggle, noclip; a key by SDL's name or X-Ray's), none to restore the tree's",
        |state: &mut GameConsoleState, argument| state.bind(GameConsoleState::BINDINGS, argument),
      ),
      ConsoleCommand::new(
        "bind_sec",
        "<action> <key>",
        "binds an action's second key, as bind does its main one",
        |state: &mut GameConsoleState, argument| state.bind(GameConsoleState::BINDINGS_SECONDARY, argument),
      ),
      ConsoleCommand::new(
        "sensitivity",
        "<radians a count>",
        "shows or sets how far a count of the mouse turns the look, in X-Ray's mouse_sens place",
        |state: &mut GameConsoleState, argument| state.set_or_show(GameConsoleState::SENSITIVITY, argument),
      ),
      ConsoleCommand::new(
        "noclip",
        "",
        "switches between walking and flying through everything",
        |state: &mut GameConsoleState, _| {
          state
            .effects
            .push(GameConsoleEffect::Movement(MovementCommand::ToggleMode));

          ConsoleAnswer::succeed(vec![String::from("Movement switched")])
        },
      ),
    ]
  }
}
