use std::num::NonZeroIsize;

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GWLP_HINSTANCE, GetClientRect, GetWindowLongPtrW, IsIconic};
use xrf_renderer::RenderWindowHost;
use xrf_renderer::raw_window_handle::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};

/// A Win32 window viewports are drawn into, read straight from its handle on the render thread.
///
/// The swapchain sits on the window itself, under the webview's child window, which shows it through the pixels the
/// page leaves transparent.
pub struct Win32WindowHost {
  hwnd: NonZeroIsize,
}

impl Win32WindowHost {
  pub fn new(hwnd: NonZeroIsize) -> Self {
    Self { hwnd }
  }

  fn get_hwnd(&self) -> HWND {
    HWND(self.hwnd.get() as *mut _)
  }
}

// SAFETY: a window asked to close while viewports draw into it is held open until the render thread answers that it
// handled their detach, and only then destroyed (`RenderState::close_window`); nothing in the application destroys a
// window any other way, and one found destroyed still drawing them is logged as a breach (`RenderState::forget_window`).
unsafe impl RenderWindowHost for Win32WindowHost {
  fn get_key(&self) -> u64 {
    self.hwnd.get() as u64
  }

  fn get_handles(&self) -> (RawWindowHandle, RawDisplayHandle) {
    let mut handle: Win32WindowHandle = Win32WindowHandle::new(self.hwnd);

    // Vulkan makes its surface from the module instance as well as the window.
    // SAFETY: plain Win32 read; a handle no longer naming a window answers zero, which is no instance.
    handle.hinstance = NonZeroIsize::new(unsafe { GetWindowLongPtrW(self.get_hwnd(), GWLP_HINSTANCE) });

    (
      RawWindowHandle::Win32(handle),
      RawDisplayHandle::Windows(WindowsDisplayHandle::new()),
    )
  }

  fn get_client_size(&self) -> (u32, u32) {
    let mut rect: RECT = RECT::default();

    // SAFETY: `rect` outlives the call; a handle no longer naming a window fails, answered as an empty area.
    match unsafe { GetClientRect(self.get_hwnd(), &mut rect) } {
      Ok(()) => (
        (rect.right - rect.left).max(0) as u32,
        (rect.bottom - rect.top).max(0) as u32,
      ),
      Err(_) => (0, 0),
    }
  }

  fn is_minimized(&self) -> bool {
    // SAFETY: plain Win32 read; a handle no longer naming a window answers false.
    unsafe { IsIconic(self.get_hwnd()) }.as_bool()
  }
}
