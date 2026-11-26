use std::time::Duration;

use sdl3::event::{Event, WindowEvent};
use sdl3::gamepad::{Axis, Button};
use sdl3::joystick::JoystickId;
use sdl3::keyboard::{Mod, Scancode};
use sdl3::mouse::{MouseButton, MouseWheelDirection};

use crate::event::platform_event::PlatformEvent;
use crate::event::platform_event_kind::PlatformEventKind;
use crate::event::platform_gamepad_axis::PlatformGamepadAxis;
use crate::event::platform_gamepad_button::PlatformGamepadButton;
use crate::event::platform_mouse_button::PlatformMouseButton;
use crate::event::platform_scancode::PlatformScancode;
use crate::window::platform_size::PlatformSize;

fn read_kind(event: Event) -> Option<PlatformEventKind> {
  PlatformEvent::read(event).map(|event| event.kind)
}

fn key_down(scancode: Scancode, repeat: bool) -> Event {
  Event::KeyDown {
    timestamp: 0,
    window_id: 1,
    keycode: None,
    scancode: Some(scancode),
    keymod: Mod::NOMOD,
    repeat,
    which: 0,
    raw: 0,
  }
}

#[test]
fn stamps_an_event_with_sdls_time() {
  let event: Option<PlatformEvent> = PlatformEvent::read(Event::Quit { timestamp: 1_500_000 });

  assert_eq!(
    event,
    Some(PlatformEvent {
      at: Duration::from_micros(1_500),
      kind: PlatformEventKind::Quit,
    })
  );
}

#[test]
fn reads_a_key_by_its_place_whatever_the_layout() {
  assert_eq!(
    read_kind(key_down(Scancode::A, false)),
    Some(PlatformEventKind::Key {
      scancode: PlatformScancode(4),
      is_down: true,
      is_repeat: false,
    })
  );
  assert_eq!(
    read_kind(key_down(Scancode::Escape, true)),
    Some(PlatformEventKind::Key {
      scancode: PlatformScancode(41),
      is_down: true,
      is_repeat: true,
    })
  );
}

#[test]
fn reads_the_drawable_size_and_whether_the_window_shows() {
  let window = |win_event: WindowEvent| Event::Window {
    timestamp: 0,
    window_id: 1,
    win_event,
  };

  assert_eq!(
    read_kind(window(WindowEvent::PixelSizeChanged(1920, 1080))),
    Some(PlatformEventKind::Resized {
      size: PlatformSize {
        width: 1920,
        height: 1080,
      },
    })
  );
  assert_eq!(
    read_kind(window(WindowEvent::Minimized)),
    Some(PlatformEventKind::Shown { is_shown: false })
  );
  assert_eq!(
    read_kind(window(WindowEvent::Restored)),
    Some(PlatformEventKind::Shown { is_shown: true })
  );
  assert_eq!(read_kind(window(WindowEvent::Moved(10, 10))), None);
}

#[test]
fn reads_a_flipped_wheel_as_the_user_turned_it() {
  let wheel = |direction: MouseWheelDirection| Event::MouseWheel {
    timestamp: 0,
    window_id: 1,
    which: 0,
    x: 0.0,
    y: 1.0,
    direction,
    mouse_x: 0.0,
    mouse_y: 0.0,
    integer_x: 0,
    integer_y: 1,
  };

  assert_eq!(
    read_kind(wheel(MouseWheelDirection::Normal)),
    Some(PlatformEventKind::MouseWheel { dx: 0.0, dy: 1.0 })
  );
  assert_eq!(
    read_kind(wheel(MouseWheelDirection::Flipped)),
    Some(PlatformEventKind::MouseWheel { dx: -0.0, dy: -1.0 })
  );
}

#[test]
fn skips_a_mouse_button_sdl_cannot_name() {
  let button = |mouse_btn: MouseButton| Event::MouseButtonDown {
    timestamp: 0,
    window_id: 1,
    which: 0,
    mouse_btn,
    clicks: 1,
    x: 5.0,
    y: 6.0,
  };

  assert_eq!(
    read_kind(button(MouseButton::X1)),
    Some(PlatformEventKind::MouseButton {
      button: PlatformMouseButton::Back,
      is_down: true,
      x: 5.0,
      y: 6.0,
    })
  );
  assert_eq!(read_kind(button(MouseButton::Unknown)), None);
}

#[test]
fn reads_a_gamepads_axis_to_one_whole_at_either_end() {
  let axis = |value: i16| Event::GamepadAxisMotion {
    timestamp: 0,
    which: JoystickId::new(7),
    axis: Axis::LeftY,
    value,
  };

  for (value, expected) in [(i16::MIN, -1.0), (0, 0.0), (i16::MAX, 1.0)] {
    assert_eq!(
      read_kind(axis(value)),
      Some(PlatformEventKind::GamepadAxis {
        gamepad: 7,
        axis: PlatformGamepadAxis(1),
        value: expected,
      })
    );
  }

  assert_eq!(
    read_kind(Event::GamepadButtonDown {
      timestamp: 0,
      which: JoystickId::new(7),
      button: Button::South,
    }),
    Some(PlatformEventKind::GamepadButton {
      gamepad: 7,
      button: PlatformGamepadButton(0),
      is_down: true,
    })
  );
}
