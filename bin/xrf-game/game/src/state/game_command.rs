use xrf_game_movement::MovementCommand;

/// Every change from outside the game's state, routed to the system it is for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameCommand {
  Movement(MovementCommand),
}
