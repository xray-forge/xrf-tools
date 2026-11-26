#![expect(unsafe_code, reason = "lends a made-up window handle, which no test presents into")]

use std::num::NonZeroIsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use xrf_renderer::RenderWindowHost;
use xrf_renderer::raw_window_handle::{
  HandleError, HasWindowHandle, RawWindowHandle, WebWindowHandle, Win32WindowHandle, WindowHandle,
};

use crate::window_host::hosted_window::HostedWindow;

struct FakeWindow {
  raw: RawWindowHandle,
  dropped: Arc<AtomicBool>,
}

impl FakeWindow {
  fn win32(dropped: &Arc<AtomicBool>) -> Self {
    Self {
      raw: RawWindowHandle::Win32(Win32WindowHandle::new(NonZeroIsize::new(0x1234).expect("non-zero"))),
      dropped: Arc::clone(dropped),
    }
  }
}

impl HasWindowHandle for FakeWindow {
  fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
    // SAFETY: the handle names no window, and no test asks a host for its surface.
    Ok(unsafe { WindowHandle::borrow_raw(self.raw) })
  }
}

impl Drop for FakeWindow {
  fn drop(&mut self) {
    self.dropped.store(true, Ordering::Release);
  }
}

#[test]
fn hosts_the_window_its_handle_names() {
  let dropped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
  let window: HostedWindow<FakeWindow> = HostedWindow::new(FakeWindow::win32(&dropped)).expect("a Win32 handle hosts");

  assert_eq!(window.get_host().get_key(), 0x1234);
  assert_eq!(window.get_host().get_key(), window.get_host().get_key());
}

#[test]
fn drops_the_window_once_no_host_is_left() {
  let dropped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
  let window: HostedWindow<FakeWindow> = HostedWindow::new(FakeWindow::win32(&dropped)).expect("a Win32 handle hosts");

  drop(window.get_host());
  drop(window);

  assert!(dropped.load(Ordering::Acquire));
}

#[test]
fn keeps_the_window_while_a_host_is_held() {
  let dropped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
  let window: HostedWindow<FakeWindow> = HostedWindow::new(FakeWindow::win32(&dropped)).expect("a Win32 handle hosts");
  let host: Arc<dyn RenderWindowHost> = window.get_host();

  drop(window);

  assert!(!dropped.load(Ordering::Acquire));

  // The window stays leaked once the host goes too: nothing is left to drop it on its own thread.
  drop(host);

  assert!(!dropped.load(Ordering::Acquire));
}

#[test]
fn refuses_a_handle_of_another_platform() {
  let dropped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
  let window: FakeWindow = FakeWindow {
    raw: RawWindowHandle::Web(WebWindowHandle::new(1)),
    dropped: Arc::clone(&dropped),
  };

  assert!(HostedWindow::new(window).is_err());
  assert!(dropped.load(Ordering::Acquire));
}
