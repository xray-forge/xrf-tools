use xrf_game_console::ConsoleFamily;

use crate::console::game_console_state::GameConsoleState;

/// The game's console context for any borrow: a line runs against `GameConsoleState<'a>`, the settings store and the
/// effects borrowed for that line.
pub(crate) struct GameConsoleFamily;

impl ConsoleFamily for GameConsoleFamily {
  type Context<'a> = GameConsoleState<'a>;
}
