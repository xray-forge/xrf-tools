use serde::{Deserialize, Serialize};

/// Which way the player moves; how each moves is the movement's [`crate::MovementModes`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementMode {
  /// A free camera through everything.
  #[default]
  Fly,
  /// On foot, on the physics world.
  Walk,
}

impl MovementMode {
  /// The other mode: walking after flight, flight after walking, the noclip toggle.
  pub fn toggled(self) -> Self {
    match self {
      Self::Fly => Self::Walk,
      Self::Walk => Self::Fly,
    }
  }
}
