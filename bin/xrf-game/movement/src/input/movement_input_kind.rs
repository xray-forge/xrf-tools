use crate::input::movement_action::MovementAction;

/// What the player did, already read through their bindings: movement never sees a key or a device.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovementInputKind {
  /// An action's key or button went down or up.
  Action { action: MovementAction, is_pressed: bool },
  /// The mouse moved, in its raw counts: right and down positive.
  Look { dx: f32, dy: f32 },
  /// Every action let go, as when the window loses focus and the releases will never arrive.
  ReleaseAll,
}
