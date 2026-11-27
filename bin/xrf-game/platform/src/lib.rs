#![doc = include_str!("../README.md")]

pub(crate) mod event;
pub(crate) mod platform;
pub(crate) mod platform_library;
pub(crate) mod platform_waker;
#[cfg(test)]
mod tests;
pub(crate) mod window;

pub use crate::event::platform_event::PlatformEvent;
pub use crate::event::platform_event_kind::PlatformEventKind;
pub use crate::event::platform_gamepad_axis::PlatformGamepadAxis;
pub use crate::event::platform_gamepad_button::PlatformGamepadButton;
pub use crate::event::platform_mouse_button::PlatformMouseButton;
pub use crate::event::platform_scancode::PlatformScancode;
pub use crate::platform::Platform;
pub use crate::platform_waker::PlatformWaker;
pub use crate::window::platform_size::PlatformSize;
pub use crate::window::platform_window::PlatformWindow;
pub use crate::window::platform_window_command::PlatformWindowCommand;
pub use crate::window::platform_window_handle::PlatformWindowHandle;
pub use crate::window::platform_window_mode::PlatformWindowMode;
pub use crate::window::platform_window_request::PlatformWindowRequest;
