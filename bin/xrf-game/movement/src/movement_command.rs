use crate::camera::movement_camera::MovementCamera;
use crate::input::movement_input::MovementInput;

/// A change from outside movement takes, between two steps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovementCommand {
  Input(MovementInput),
  /// Stands the camera where it is told, as the control endpoint's pose does.
  SetCamera(MovementCamera),
}
