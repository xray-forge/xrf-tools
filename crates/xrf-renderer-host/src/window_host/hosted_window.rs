#![expect(
  unsafe_code,
  reason = "builds a host over the window it owns, kept alive while any host of it is held, as the builder requires"
)]

use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::Arc;

use xrf_renderer::RenderWindowHost;
use xrf_renderer::raw_window_handle::HasWindowHandle;
#[cfg(windows)]
use xrf_renderer::raw_window_handle::RawWindowHandle;

#[cfg(windows)]
use crate::window_host::window_host_builder::WindowHostBuilder;

/// A window viewports are drawn into, owned for as long as the render thread may present into it: the safe way to host
/// a window this process owns, where `WindowHostBuilder` takes the caller's word for a window it only names.
///
/// Drop it on the window's own thread once `RenderQueue::wait_handled` answered the detach of every viewport drawn into
/// it: it drops the window there. Dropped while a host of it is still held, it leaks the window instead, and logs so.
pub struct HostedWindow<W: HasWindowHandle> {
  window: ManuallyDrop<W>,
  host: Arc<dyn RenderWindowHost>,
}

impl<W: HasWindowHandle> HostedWindow<W> {
  /// Holds `window`, reading its native handle once: a window's handle stays the same for its life.
  ///
  /// # Errors
  ///
  /// Returns why the window cannot be drawn in: its handle is unavailable, or of a platform no host is made for here.
  pub fn new(window: W) -> Result<Self, String> {
    let host: Arc<dyn RenderWindowHost> = Self::create_host(&window)?;

    Ok(Self {
      window: ManuallyDrop::new(window),
      host,
    })
  }

  /// The host to attach this window's viewports with.
  pub fn get_host(&self) -> Arc<dyn RenderWindowHost> {
    Arc::clone(&self.host)
  }

  #[cfg(windows)]
  fn create_host(window: &W) -> Result<Arc<dyn RenderWindowHost>, String> {
    match window
      .window_handle()
      .map_err(|error| format!("The window's handle is not available: {error}"))?
      .as_raw()
    {
      // SAFETY: the window this handle names is owned by the value being made, which drops it only once no host of it
      // is left, so it lives past the detach of every viewport drawn into it.
      RawWindowHandle::Win32(handle) => unsafe { WindowHostBuilder::new(handle.hwnd) }.build(),
      other => Err(format!("A Windows window has a Win32 handle, not {other:?}")),
    }
  }

  #[cfg(not(windows))]
  fn create_host(_: &W) -> Result<Arc<dyn RenderWindowHost>, String> {
    // todo: Host a window this process owns on macOS and Linux, from its AppKit, X11 or Wayland handles.
    Err("Hosting a window the process owns is supported on Windows only".to_string())
  }
}

impl<W: HasWindowHandle> Deref for HostedWindow<W> {
  type Target = W;

  fn deref(&self) -> &W {
    &self.window
  }
}

impl<W: HasWindowHandle> Drop for HostedWindow<W> {
  fn drop(&mut self) {
    // Only this value hands out hosts, so a count of one is final: no host can come back to present into the window.
    if Arc::strong_count(&self.host) > 1 {
      log::error!("A window was let go while the renderer may still draw into it, so it is kept open until exit");

      return;
    }

    // SAFETY: the window is dropped once, here, and never read after.
    unsafe { ManuallyDrop::drop(&mut self.window) };
  }
}
