use crate::event::platform_gamepad_axis::PlatformGamepadAxis;
use crate::event::platform_gamepad_button::PlatformGamepadButton;
use crate::event::platform_mouse_button::PlatformMouseButton;
use crate::event::platform_scancode::PlatformScancode;
use crate::window::platform_size::PlatformSize;

/// What happened. Pointer positions are in the window's logical units from its top-left corner; motion is the device's
/// own, unaccelerated where the cursor is grabbed.
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformEventKind {
  /// The window's close button or the system asked the game to quit.
  Quit,
  /// The window's drawable area is now `size` device pixels.
  Resized {
    size: PlatformSize,
  },
  /// The window shows nothing now, minimised or hidden, or shows again.
  Shown {
    is_shown: bool,
  },
  Focused {
    is_focused: bool,
  },
  /// A key by its physical place on the keyboard, whatever the layout prints on it.
  Key {
    scancode: PlatformScancode,
    is_down: bool,
    is_repeat: bool,
  },
  /// Text typed while the window accepts text, after the layout and any input method.
  Text {
    text: String,
  },
  MouseMoved {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
  },
  MouseButton {
    button: PlatformMouseButton,
    is_down: bool,
    x: f32,
    y: f32,
  },
  /// A wheel's turn, in notches: up and right are positive.
  MouseWheel {
    dx: f32,
    dy: f32,
  },
  /// A gamepad was connected; `gamepad` names it in its later events until it is removed.
  GamepadAdded {
    gamepad: u32,
  },
  GamepadRemoved {
    gamepad: u32,
  },
  GamepadButton {
    gamepad: u32,
    button: PlatformGamepadButton,
    is_down: bool,
  },
  /// An axis moved, to -1 to 1 for a stick and 0 to 1 for a trigger.
  GamepadAxis {
    gamepad: u32,
    axis: PlatformGamepadAxis,
    value: f32,
  },
}
