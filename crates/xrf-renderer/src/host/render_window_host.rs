/// A native window viewports are drawn into, as the application hosting the renderer knows it.
///
/// The renderer polls it rather than being told of changes: the client size each frame, so the swapchain follows a
/// resize at once, and whether the window shows anything, which a webview inside it is never told.
///
/// # Safety
///
/// The target `get_surface_target` answers must name a live window, or a layer drawn in one, until every viewport
/// attached with this host is detached and the render thread has handled that detach, as `RenderQueue::wait_handled`
/// answering anything but a timeout tells: the renderer presents into it until then.
pub unsafe trait RenderWindowHost: Send + Sync + 'static {
  /// What tells this window apart from every other, stable for its life.
  fn get_key(&self) -> u64;

  /// What the render thread makes the window's surface from, on the render thread: the window's own handles where the
  /// platform's API makes a surface from them on any thread, else what it does make one from.
  fn get_surface_target(&self) -> wgpu::SurfaceTargetUnsafe;

  /// The client area, in device pixels.
  fn get_client_size(&self) -> (u32, u32);

  /// Whether the window shows nothing at all now: minimised, or wherever the platform tells, covered entirely.
  fn is_hidden(&self) -> bool;
}
