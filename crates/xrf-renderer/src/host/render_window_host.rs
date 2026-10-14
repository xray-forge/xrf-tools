use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

/// A native window viewports are drawn into, as the application hosting the renderer knows it.
///
/// The renderer polls it rather than being told of changes: the client size each frame, so the swapchain follows a
/// resize at once, and whether the window is minimised, which a webview inside it is never told.
///
/// # Safety
///
/// The handles `get_handles` answers must name a live window until every viewport attached with this host is detached
/// and the render thread has handled that detach, as `RenderQueue::wait_handled` answering anything but a timeout
/// tells: the renderer presents into the window through them until then.
pub unsafe trait RenderWindowHost: Send + Sync + 'static {
  /// What tells this window apart from every other, stable for its life.
  fn get_key(&self) -> u64;

  /// The handles a surface is made from.
  fn get_handles(&self) -> (RawWindowHandle, RawDisplayHandle);

  /// The client area, in device pixels.
  fn get_client_size(&self) -> (u32, u32);

  /// Whether the window shows nothing at all now.
  fn is_minimized(&self) -> bool;
}
