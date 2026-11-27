use crate::mode::fly_mode::FlyMode;
use crate::mode::walk_mode::WalkMode;

/// How the player moves: what turns their input into motion each step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovementMode {
  /// A free camera through everything.
  Fly(FlyMode),
  /// On foot, on the physics world.
  Walk(WalkMode),
}

impl MovementMode {
  /// The other mode, as it starts: walking after flight, flight after walking, the noclip toggle.
  pub fn toggled(&self) -> Self {
    match self {
      Self::Fly(_) => Self::Walk(WalkMode::default()),
      Self::Walk(_) => Self::Fly(FlyMode::default()),
    }
  }
}

impl Default for MovementMode {
  fn default() -> Self {
    Self::Fly(FlyMode::default())
  }
}
