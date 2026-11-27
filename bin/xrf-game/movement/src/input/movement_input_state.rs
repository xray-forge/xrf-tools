use crate::input::movement_action::MovementAction;
use crate::input::movement_input_kind::MovementInputKind;

/// What the player holds and whether they toggled sprinting on: what a movement mode reads each step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MovementInputState {
  held: [bool; MovementAction::ALL.len()],
  is_sprinting: bool,
}

impl MovementInputState {
  pub fn is_held(&self, action: MovementAction) -> bool {
    self.held[action.get_index()]
  }

  /// `1` while `positive` alone is held, `-1` while `negative` alone is, else `0`.
  pub fn get_axis(&self, positive: MovementAction, negative: MovementAction) -> f32 {
    f32::from(i8::from(self.is_held(positive)) - i8::from(self.is_held(negative)))
  }

  /// Whether sprinting is toggled on.
  pub fn is_sprinting(&self) -> bool {
    self.is_sprinting
  }

  /// Turns sprinting off, as stopping does.
  pub fn stop_sprinting(&mut self) {
    self.is_sprinting = false;
  }

  pub(crate) fn apply(&mut self, kind: MovementInputKind) {
    match kind {
      MovementInputKind::Action {
        action: MovementAction::Sprint,
        is_pressed,
      } => {
        if is_pressed && !self.is_held(MovementAction::Sprint) {
          self.is_sprinting = !self.is_sprinting;
        }

        self.held[MovementAction::Sprint.get_index()] = is_pressed;
      }
      MovementInputKind::Action { action, is_pressed } => self.held[action.get_index()] = is_pressed,
      MovementInputKind::ReleaseAll => self.held = [false; MovementAction::ALL.len()],
    }
  }
}
