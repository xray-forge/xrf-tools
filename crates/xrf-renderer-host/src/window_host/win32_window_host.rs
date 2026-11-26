#![expect(
  unsafe_code,
  reason = "implements the renderer's unsafe window-host contract over Win32 window handles"
)]

use std::num::NonZeroIsize;

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GWLP_HINSTANCE, GetClientRect, GetWindowLongPtrW, IsIconic};
use xrf_renderer::raw_window_handle::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};
use xrf_renderer::{RenderWindowHost, SurfaceTargetUnsafe};

/// A Win32 window viewports are drawn into, read straight from its handle on the render thread.
///
/// The swapchain sits on the window itself, under any child window it has: a webview's shows it through the pixels its
/// page leaves transparent.
pub(super) struct Win32WindowHost {
  hwnd: NonZeroIsize,
}

impl Win32WindowHost {
  /// A host over the window `hwnd` names.
  pub(super) fn new(hwnd: NonZeroIsize) -> Self {
    Self { hwnd }
  }

  fn get_hwnd(&self) -> HWND {
    HWND(self.hwnd.get() as *mut _)
  }
}

// SAFETY: the host is made only through `WindowHostBuilder`, whose constructor has its caller promise the window lives
// until the render thread handled the detach of every viewport drawn into it.
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
    match unsafe { GetClientRect(self.get_hwnd(), &raw mut rect) } {
      Ok(()) => (
        (rect.right - rect.left).max(0).cast_unsigned(),
        (rect.bottom - rect.top).max(0).cast_unsigned(),
      ),
      Err(_) => (0, 0),
    }
  }

  fn is_hidden(&self) -> bool {
    // SAFETY: plain Win32 read; a handle no longer naming a window answers false.
    unsafe { IsIconic(self.get_hwnd()) }.as_bool()
  }
}
