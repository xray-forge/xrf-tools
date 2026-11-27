use xrf_game_movement::MovementCommand;

/// What a console line asks of the running game beyond the settings, which it sets in the store: applied by play after
/// the line runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GameConsoleEffect {
  /// A command for the player's movement, which the game loop routes as it routes the player's input.
  Movement(MovementCommand),
  Quit,
}
