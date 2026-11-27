use std::time::Duration;

use xrf_game_movement::{MovementAction, MovementCommand, MovementInput, MovementInputKind};
use xrf_game_platform::{PlatformEvent, PlatformEventKind, PlatformScancode};

use crate::input::game_bindings::GameBindings;
use crate::state::game_command::GameCommand;

const AT: Duration = Duration::from_millis(42);

fn event(kind: PlatformEventKind) -> PlatformEvent {
  PlatformEvent { at: AT, kind }
}

fn key(scancode: u16, is_down: bool, is_repeat: bool) -> PlatformEvent {
  event(PlatformEventKind::Key {
    scancode: PlatformScancode(scancode),
    is_down,
    is_repeat,
  })
}

fn movement(kind: MovementInputKind) -> Option<GameCommand> {
  Some(GameCommand::Movement(MovementCommand::Input(MovementInput {
    at: AT,
    kind,
  })))
}

#[test]
fn reads_a_bound_key_by_its_place_and_skips_its_repeats() {
  let bindings: GameBindings = GameBindings::default();

  assert_eq!(
    bindings.read(&key(26, true, false)),
    movement(MovementInputKind::Action {
      action: MovementAction::Forward,
      is_pressed: true,
    })
  );
  assert_eq!(
    bindings.read(&key(225, false, false)),
    movement(MovementInputKind::Action {
      action: MovementAction::Fast,
      is_pressed: false,
    })
  );
  assert_eq!(bindings.read(&key(26, true, true)), None);
  // F12 is bound to nothing.
  assert_eq!(bindings.read(&key(69, true, false)), None);
}

#[test]
fn reads_the_mouse_as_a_look_and_losing_focus_as_letting_go() {
  let bindings: GameBindings = GameBindings::default();

  assert_eq!(
    bindings.read(&event(PlatformEventKind::MouseMoved {
      x: 10.0,
      y: 20.0,
      dx: 3.0,
      dy: -2.0,
    })),
    movement(MovementInputKind::Look { dx: 3.0, dy: -2.0 })
  );
  assert_eq!(
    bindings.read(&event(PlatformEventKind::Focused { is_focused: false })),
    movement(MovementInputKind::ReleaseAll)
  );
  assert_eq!(
    bindings.read(&event(PlatformEventKind::Focused { is_focused: true })),
    None
  );
}
