use std::time::Duration;

use sdl3::event::{Event, WindowEvent};
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use sdl3::mouse::{MouseButton, MouseWheelDirection};

use crate::event::platform_event::PlatformEvent;
use crate::event::platform_event_kind::PlatformEventKind;
use crate::event::platform_gamepad_axis::PlatformGamepadAxis;
use crate::event::platform_gamepad_button::PlatformGamepadButton;
use crate::event::platform_mouse_button::PlatformMouseButton;
use crate::event::platform_scancode::PlatformScancode;
use crate::window::platform_size::PlatformSize;

impl PlatformEvent {
  /// The game's event for one of SDL's, none for those the game does not act on.
  pub(crate) fn read(event: Event) -> Option<Self> {
    let at: Duration = Duration::from_nanos(event.get_timestamp());
    let kind: PlatformEventKind = match event {
      Event::Quit { .. } => PlatformEventKind::Quit,
      Event::Window { win_event, .. } => Self::read_window(win_event)?,
      Event::KeyDown {
        scancode: Some(scancode),
        repeat,
        ..
      } => PlatformEventKind::Key {
        scancode: Self::read_scancode(scancode)?,
        is_down: true,
        is_repeat: repeat,
      },
      Event::KeyUp {
        scancode: Some(scancode),
        ..
      } => PlatformEventKind::Key {
        scancode: Self::read_scancode(scancode)?,
        is_down: false,
        is_repeat: false,
      },
      Event::TextInput { text, .. } => PlatformEventKind::Text { text },
      Event::MouseMotion { x, y, xrel, yrel, .. } => PlatformEventKind::MouseMoved {
        x,
        y,
        dx: xrel,
        dy: yrel,
      },
      Event::MouseButtonDown { mouse_btn, x, y, .. } => PlatformEventKind::MouseButton {
        button: Self::read_mouse_button(mouse_btn)?,
        is_down: true,
        x,
        y,
      },
      Event::MouseButtonUp { mouse_btn, x, y, .. } => PlatformEventKind::MouseButton {
        button: Self::read_mouse_button(mouse_btn)?,
        is_down: false,
        x,
        y,
      },
      Event::MouseWheel { x, y, direction, .. } => match direction {
        MouseWheelDirection::Flipped => PlatformEventKind::MouseWheel { dx: -x, dy: -y },
        _ => PlatformEventKind::MouseWheel { dx: x, dy: y },
      },
      Event::GamepadAdded { which, .. } => PlatformEventKind::GamepadAdded { gamepad: which.raw() },
      Event::GamepadRemoved { which, .. } => PlatformEventKind::GamepadRemoved { gamepad: which.raw() },
      Event::GamepadButtonDown { which, button, .. } => PlatformEventKind::GamepadButton {
        gamepad: which.raw(),
        button: Self::read_gamepad_button(button)?,
        is_down: true,
      },
      Event::GamepadButtonUp { which, button, .. } => PlatformEventKind::GamepadButton {
        gamepad: which.raw(),
        button: Self::read_gamepad_button(button)?,
        is_down: false,
      },
      Event::GamepadAxisMotion { which, axis, value, .. } => PlatformEventKind::GamepadAxis {
        gamepad: which.raw(),
        axis: Self::read_gamepad_axis(axis)?,
        value: (f32::from(value) / f32::from(i16::MAX)).max(-1.0),
      },
      _ => return None,
    };

    Some(Self { at, kind })
  }

  fn read_window(event: WindowEvent) -> Option<PlatformEventKind> {
    match event {
      WindowEvent::PixelSizeChanged(width, height) => Some(PlatformEventKind::Resized {
        size: PlatformSize {
          width: u32::try_from(width).unwrap_or(0),
          height: u32::try_from(height).unwrap_or(0),
        },
      }),
      WindowEvent::Minimized | WindowEvent::Hidden => Some(PlatformEventKind::Shown { is_shown: false }),
      WindowEvent::Restored | WindowEvent::Maximized | WindowEvent::Shown => {
        Some(PlatformEventKind::Shown { is_shown: true })
      }
      WindowEvent::FocusGained => Some(PlatformEventKind::Focused { is_focused: true }),
      WindowEvent::FocusLost => Some(PlatformEventKind::Focused { is_focused: false }),
      _ => None,
    }
  }

  fn read_scancode(scancode: Scancode) -> Option<PlatformScancode> {
    u16::try_from(scancode as i32).ok().map(PlatformScancode)
  }

  fn read_mouse_button(button: MouseButton) -> Option<PlatformMouseButton> {
    match button {
      MouseButton::Left => Some(PlatformMouseButton::Left),
      MouseButton::Middle => Some(PlatformMouseButton::Middle),
      MouseButton::Right => Some(PlatformMouseButton::Right),
      MouseButton::X1 => Some(PlatformMouseButton::Back),
      MouseButton::X2 => Some(PlatformMouseButton::Forward),
      MouseButton::Unknown => None,
    }
  }

  fn read_gamepad_button(button: Button) -> Option<PlatformGamepadButton> {
    u8::try_from(button as i32).ok().map(PlatformGamepadButton)
  }

  fn read_gamepad_axis(axis: Axis) -> Option<PlatformGamepadAxis> {
    u8::try_from(axis as i32).ok().map(PlatformGamepadAxis)
  }
}
