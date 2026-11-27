use crate::mode::fly_mode::FlyMode;

/// How the player moves: what turns their input into motion each step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovementMode {
  /// A free camera through everything; walking on a physics engine comes later.
  Fly(FlyMode),
}

impl Default for MovementMode {
  fn default() -> Self {
    Self::Fly(FlyMode::default())
  }
}
