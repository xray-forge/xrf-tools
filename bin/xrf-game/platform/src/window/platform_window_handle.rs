use sdl3::video::Window;
use xrf_renderer::raw_window_handle::{HandleError, HasWindowHandle, WindowHandle};

/// The game window as the renderer's host holds it: a clone of SDL's window, which keeps it open while held.
pub struct PlatformWindowHandle {
  window: Window,
}

impl PlatformWindowHandle {
  pub(crate) fn new(window: Window) -> Self {
    Self { window }
  }
}

impl HasWindowHandle for PlatformWindowHandle {
  fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
    self.window.window_handle()
  }
}
