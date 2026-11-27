use crate::event::movement_step::MovementStep;

/// What movement tells presentation once, in the order it happened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovementEvent {
  Step(MovementStep),
}
