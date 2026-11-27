use serde::{Deserialize, Serialize};

use crate::input::movement_action::MovementAction;

/// What the player did, already read through their bindings: movement never sees a key or a device. Where they look is
/// presentation's, sent as a heading (`MovementCommand::Face`), never as the mouse's counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementInputKind {
  /// An action's key or button went down or up.
  Action { action: MovementAction, is_pressed: bool },
  /// Every action let go, as when the window loses focus and the releases will never arrive.
  ReleaseAll,
}
