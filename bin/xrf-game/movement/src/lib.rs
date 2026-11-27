#![doc = include_str!("../README.md")]

pub(crate) mod camera;
pub(crate) mod input;
pub(crate) mod mode;
pub(crate) mod movement;
pub(crate) mod movement_command;
pub(crate) mod movement_context;

#[cfg(test)]
mod tests;

pub use crate::camera::movement_camera::MovementCamera;
pub use crate::camera::movement_heading::MovementHeading;
pub use crate::input::movement_action::MovementAction;
pub use crate::input::movement_input::MovementInput;
pub use crate::input::movement_input_kind::MovementInputKind;
pub use crate::input::movement_input_state::MovementInputState;
pub use crate::mode::fly_mode::FlyMode;
pub use crate::mode::movement_footing::MovementFooting;
pub use crate::mode::movement_mode::MovementMode;
pub use crate::mode::movement_modes::MovementModes;
pub use crate::mode::walk_mode::WalkMode;
pub use crate::mode::walk_pace::WalkPace;
pub use crate::mode::walk_stance::WalkStance;
pub use crate::mode::walk_state::WalkState;
pub use crate::movement::Movement;
pub use crate::movement_command::MovementCommand;
pub use crate::movement_context::MovementContext;
