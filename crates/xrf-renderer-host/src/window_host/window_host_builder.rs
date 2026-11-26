#![expect(
  unsafe_code,
  reason = "its constructor takes the window's lifetime as the caller's promise, which the hosts' unsafe contract rests on"
)]

use std::num::NonZeroIsize;
use std::sync::Arc;

use xrf_renderer::RenderWindowHost;

#[cfg(target_os = "macos")]
use crate::window_host::macos_window_host::MacosWindowHost;
#[cfg(windows)]
use crate::window_host::win32_window_host::Win32WindowHost;

/// Builds the host viewports draw into from a window's native handle: one host for each platform the renderer presents
/// on.
pub struct WindowHostBuilder {
  handle: NonZeroIsize,
}

impl WindowHostBuilder {
  /// A builder of hosts over the native window `handle` names: its `HWND` on Windows, its `NSWindow` on macOS.
  ///
  /// # Safety
  ///
  /// `handle` names a live window of this process, and the caller keeps that window alive until the render thread has
  /// handled the detach of every viewport drawn into the host built from it.
  pub unsafe fn new(handle: NonZeroIsize) -> Self {
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

  /// The host of the window whose `NSWindow` the handle is, its renderer view placed under the window's other views.
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
    // todo: Host native viewports on Linux, from a window's X11 or Wayland handles.
    Err("Native viewports are drawn on Windows and macOS only".to_string())
  }
}
