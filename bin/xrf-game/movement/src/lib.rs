#![doc = include_str!("../README.md")]

pub(crate) mod camera;
pub(crate) mod input;
pub(crate) mod mode;
pub(crate) mod movement;
pub(crate) mod movement_command;

#[cfg(test)]
mod tests;

pub use crate::camera::movement_camera::MovementCamera;
pub use crate::camera::movement_heading::MovementHeading;
pub use crate::input::movement_action::MovementAction;
pub use crate::input::movement_input::MovementInput;
pub use crate::input::movement_input_kind::MovementInputKind;
pub use crate::input::movement_input_state::MovementInputState;
pub use crate::mode::fly_mode::FlyMode;
pub use crate::mode::movement_mode::MovementMode;
pub use crate::movement::Movement;
pub use crate::movement_command::MovementCommand;
