use crate::mode::fly_mode::FlyMode;
use crate::mode::walk_mode::WalkMode;

/// How each mode moves, kept whichever the player is in, so switching between them keeps what the game read for each:
/// walking's from the tree's `[actor]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MovementModes {
  pub fly: FlyMode,
  pub walk: WalkMode,
}
