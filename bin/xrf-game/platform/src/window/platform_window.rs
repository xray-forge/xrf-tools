use sdl3::keyboard::TextInputUtil;
use sdl3::mouse::MouseUtil;
use sdl3::video::Window;
use xrf_renderer_host::HostedWindow;

use crate::window::platform_size::PlatformSize;
use crate::window::platform_window_command::PlatformWindowCommand;
use crate::window::platform_window_handle::PlatformWindowHandle;
use crate::window::platform_window_mode::PlatformWindowMode;

/// The game's window, kept on the thread that opened it, as SDL requires.
pub struct PlatformWindow {
  window: Window,
  mouse: MouseUtil,
  text: TextInputUtil,
}

impl PlatformWindow {
  pub(crate) fn new(window: Window, mouse: MouseUtil, text: TextInputUtil) -> Self {
    Self { window, mouse, text }
  }

  /// The window as the renderer's viewports are attached to it: drop it on this thread once their detach is handled,
  /// as `HostedWindow` says.
  ///
  /// # Errors
  ///
  /// Returns why the window cannot be drawn in, as `HostedWindow::new` does.
  pub fn create_hosted(&self) -> Result<HostedWindow<PlatformWindowHandle>, String> {
    HostedWindow::new(PlatformWindowHandle::new(self.window.clone()))
  }

  /// The drawable area, in device pixels.
  pub fn get_size(&self) -> PlatformSize {
    let (width, height): (u32, u32) = self.window.size_in_pixels();

    PlatformSize { width, height }
  }

  /// Device pixels per logical unit on the display the window is on.
  pub fn get_scale(&self) -> f32 {
    self.window.display_scale()
  }

  /// Does what `command` asks.
  ///
  /// # Errors
  ///
  /// Returns why SDL refused it.
  pub fn apply(&mut self, command: PlatformWindowCommand) -> Result<(), String> {
    match command {
      PlatformWindowCommand::SetMode { mode } => self
        .window
        .set_fullscreen(mode == PlatformWindowMode::Fullscreen)
        .map_err(|error| format!("The window's mode cannot be set to {mode:?}: {error}")),
      PlatformWindowCommand::SetSize { size } => self
        .window
        .set_size(size.width, size.height)
        .map_err(|error| format!("The window cannot be sized {}x{}: {error}", size.width, size.height)),
      PlatformWindowCommand::GrabCursor { is_grabbed } => {
        self.mouse.set_relative_mouse_mode(&self.window, is_grabbed);

        Ok(())
      }
      PlatformWindowCommand::AcceptText { is_accepting } => {
        if is_accepting {
          self.text.start(&self.window);
        } else {
          self.text.stop(&self.window);
        }

        Ok(())
      }
    }
  }
}
