use std::collections::HashMap;

use xrf_game_movement::{MovementAction, MovementCommand, MovementInput, MovementInputKind};
use xrf_game_platform::{PlatformEvent, PlatformEventKind, PlatformScancode};

use crate::state::game_command::GameCommand;

/// Which key does what: the platform's events read into the game's commands. Keys are bound by their place on the
/// keyboard, so the layout never moves them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameBindings {
  keys: HashMap<PlatformScancode, MovementAction>,
}

impl Default for GameBindings {
  /// Call of Pripyat's own (`gamedata/configs/default_controls.ltx`): W and S as `forward` and `back`, A and D as
  /// `lstrafe` and `rstrafe`, Space up as `jump`, Left Ctrl down as `crouch`, Left Shift faster as `accel`.
  fn default() -> Self {
    Self {
      keys: HashMap::from([
        (PlatformScancode(26), MovementAction::Forward),
        (PlatformScancode(22), MovementAction::Back),
        (PlatformScancode(4), MovementAction::Left),
        (PlatformScancode(7), MovementAction::Right),
        (PlatformScancode(44), MovementAction::Up),
        (PlatformScancode(224), MovementAction::Down),
        (PlatformScancode(225), MovementAction::Fast),
      ]),
    }
  }
}

impl GameBindings {
  /// The command an event asks for, none for an event no binding reads. A key's repeats are not presses; losing focus
  /// lets every action go, since its releases will never arrive.
  pub fn read(&self, event: &PlatformEvent) -> Option<GameCommand> {
    let kind: MovementInputKind = match &event.kind {
      PlatformEventKind::Key {
        scancode,
        is_down,
        is_repeat: false,
      } => MovementInputKind::Action {
        action: *self.keys.get(scancode)?,
        is_pressed: *is_down,
      },
      PlatformEventKind::MouseMoved { dx, dy, .. } => MovementInputKind::Look { dx: *dx, dy: *dy },
      PlatformEventKind::Focused { is_focused: false } => MovementInputKind::ReleaseAll,
      _ => return None,
    };

    Some(GameCommand::Movement(MovementCommand::Input(MovementInput {
      at: event.at,
      kind,
    })))
  }
}
