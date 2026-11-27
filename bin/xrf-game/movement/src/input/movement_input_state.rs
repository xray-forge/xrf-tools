use xrf_math::glam::Vec2;

use crate::input::movement_action::MovementAction;
use crate::input::movement_input_kind::MovementInputKind;

/// What the player holds and how far they looked since the last tick took it: what a movement mode reads each step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MovementInputState {
  held: [bool; MovementAction::ALL.len()],
  look: Vec2,
}

impl MovementInputState {
  pub fn is_held(&self, action: MovementAction) -> bool {
    self.held[action.get_index()]
  }

  /// `1` while `positive` alone is held, `-1` while `negative` alone is, else `0`.
  pub fn get_axis(&self, positive: MovementAction, negative: MovementAction) -> f32 {
    f32::from(i8::from(self.is_held(positive)) - i8::from(self.is_held(negative)))
  }

  /// The mouse's counts since the last take, which starts them again from nothing.
  pub fn take_look(&mut self) -> Vec2 {
    std::mem::take(&mut self.look)
  }

  pub(crate) fn apply(&mut self, kind: MovementInputKind) {
    match kind {
      MovementInputKind::Action { action, is_pressed } => self.held[action.get_index()] = is_pressed,
      MovementInputKind::Look { dx, dy } => self.look += Vec2::new(dx, dy),
      MovementInputKind::ReleaseAll => self.held = [false; MovementAction::ALL.len()],
    }
  }
}
