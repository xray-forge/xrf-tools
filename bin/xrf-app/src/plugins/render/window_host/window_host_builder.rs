use std::num::NonZeroIsize;
use std::sync::Arc;

use xrf_renderer::RenderWindowHost;

#[cfg(target_os = "macos")]
use crate::plugins::render::window_host::macos_window_host::MacosWindowHost;
#[cfg(windows)]
use crate::plugins::render::window_host::win32_window_host::Win32WindowHost;

/// Builds the host viewports draw into from a window's native handle, as `WindowHandles` recorded it: one host for each
/// platform the renderer presents on.
pub struct WindowHostBuilder {
  handle: NonZeroIsize,
}

impl WindowHostBuilder {
  pub fn new(handle: NonZeroIsize) -> Self {
    Self { handle }
  }

  /// The host of the window whose HWND the handle is.
  ///
  /// # Errors
  ///
  /// Never on Windows; the signature is every platform's.
  #[cfg(windows)]
  pub fn build(self) -> Result<Arc<dyn RenderWindowHost>, String> {
    Ok(Arc::new(Win32WindowHost::new(self.handle)))
  }

  /// The host of the window whose `NSWindow` the handle is, its renderer view placed under the webview.
  ///
  /// # Errors
  ///
  /// Returns why the window cannot be drawn in, as `MacosWindowHost::attach` does.
  #[cfg(target_os = "macos")]
  pub fn build(self) -> Result<Arc<dyn RenderWindowHost>, String> {
    Ok(Arc::new(MacosWindowHost::attach(self.handle)?))
  }

  /// No host: native viewports are not drawn on this platform.
  ///
  /// # Errors
  ///
  /// Always.
  #[cfg(not(any(windows, target_os = "macos")))]
  pub fn build(self) -> Result<Arc<dyn RenderWindowHost>, String> {
    // todo: Host native viewports on Linux, where the webview's window composes differently.
    Err("Native viewports are drawn on Windows and macOS only".to_string())
  }
}
