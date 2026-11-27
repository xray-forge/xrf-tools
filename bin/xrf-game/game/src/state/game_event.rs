use xrf_game_alife::AlifeEvent;
use xrf_game_movement::MovementEvent;

/// What the game's state tells presentation once, in the order its systems told it, each routed from the system that
/// told it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameEvent {
  Movement(MovementEvent),
  Alife(AlifeEvent),
}
