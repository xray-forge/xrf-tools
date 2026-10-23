use std::num::NonZeroIsize;

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GWLP_HINSTANCE, GetClientRect, GetWindowLongPtrW, IsIconic};
use xrf_renderer::raw_window_handle::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};
use xrf_renderer::{RenderWindowHost, SurfaceTargetUnsafe};

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

// SAFETY: a window whose close its page approved, while viewports draw into it, is held open until the render thread
// answers that it handled their detach, and only then destroyed (`RenderViewports::close_window`). Only a page draws
// viewports, and a page that draws them asks before its window closes, so no close goes ahead unapproved while they
// draw; nothing in the application destroys a window any other way, and one found destroyed still drawing them is
// logged as a breach (`RenderViewports::forget_window`).
unsafe impl RenderWindowHost for Win32WindowHost {
  fn get_key(&self) -> u64 {
    self.hwnd.get() as u64
  }

  fn get_surface_target(&self) -> SurfaceTargetUnsafe {
    let mut handle: Win32WindowHandle = Win32WindowHandle::new(self.hwnd);

    // Vulkan makes its surface from the module instance as well as the window.
    // SAFETY: plain Win32 read; a handle no longer naming a window answers zero, which is no instance.
    handle.hinstance = NonZeroIsize::new(unsafe { GetWindowLongPtrW(self.get_hwnd(), GWLP_HINSTANCE) });

    // Both D3D12 and Vulkan make a surface from a window's handles on any thread.
    SurfaceTargetUnsafe::RawHandle {
      raw_display_handle: Some(RawDisplayHandle::Windows(WindowsDisplayHandle::new())),
      raw_window_handle: RawWindowHandle::Win32(handle),
    }
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

  fn is_hidden(&self) -> bool {
    // SAFETY: plain Win32 read; a handle no longer naming a window answers false.
    unsafe { IsIconic(self.get_hwnd()) }.as_bool()
  }
}
