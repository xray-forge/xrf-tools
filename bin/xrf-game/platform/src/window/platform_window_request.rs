use crate::window::platform_size::PlatformSize;
use crate::window::platform_window_mode::PlatformWindowMode;

/// The window to open.
#[derive(Clone, Debug)]
pub struct PlatformWindowRequest {
  pub title: String,
  /// Its size while windowed, in the display's logical units: device pixels over its scale.
  pub size: PlatformSize,
  pub mode: PlatformWindowMode,
}
