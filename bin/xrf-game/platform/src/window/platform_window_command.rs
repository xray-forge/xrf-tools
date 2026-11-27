use crate::window::platform_size::PlatformSize;
use crate::window::platform_window_mode::PlatformWindowMode;

/// What the game tells its window to do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformWindowCommand {
  /// Shows the window, opened hidden, and raises it.
  Show,
  SetMode {
    mode: PlatformWindowMode,
  },
  /// Its size while windowed, in the display's logical units.
  SetSize {
    size: PlatformSize,
  },
  /// Hides the cursor and keeps it in the window, motion then reported as the device's own: a first-person view's.
  GrabCursor {
    is_grabbed: bool,
  },
  /// Reports typed text, starting the system's input method where one is set up, as a console's field needs.
  AcceptText {
    is_accepting: bool,
  },
}
