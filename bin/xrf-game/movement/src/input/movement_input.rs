use std::time::Duration;

use crate::input::movement_input_kind::MovementInputKind;

/// One thing the player did, and when, on the platform's clock its events carry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementInput {
  pub at: Duration,
  pub kind: MovementInputKind,
}
