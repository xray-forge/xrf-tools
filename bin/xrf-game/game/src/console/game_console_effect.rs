use xrf_game_movement::MovementCommand;

/// What a console line asks of the running game beyond the console's own state, applied by play after the line runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GameConsoleEffect {
  /// The settings changed: the view draws with them from the next frame.
  ApplySettings,
  /// A command for the player's movement, which the game loop routes as it routes the player's input.
  Movement(MovementCommand),
  Quit,
}
