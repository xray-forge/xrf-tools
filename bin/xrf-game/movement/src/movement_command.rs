use serde::{Deserialize, Serialize};

use crate::camera::movement_camera::MovementCamera;
use crate::camera::movement_heading::MovementHeading;
use crate::input::movement_input::MovementInput;

/// A change from outside movement takes, between two steps.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementCommand {
  Input(MovementInput),
  /// Faces the player where presentation turned the view, absolute: movement walks and flies along it, never turning
  /// the view itself.
  Face(MovementHeading),
  /// Stands the camera where it is told, as the control endpoint's pose does.
  SetCamera(MovementCamera),
  /// Walks after flight and flies after walking, as an engine's noclip does.
  ToggleMode,
}
