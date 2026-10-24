use std::collections::HashMap;
use std::num::NonZeroIsize;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use tauri::Runtime;
use tauri::webview::WebviewWindow;

/// The native handle of every window that can host a native viewport, by its label, recorded as each is built: its HWND
/// on Windows, its `NSWindow` on macOS, which the render plugin's window hosts draw through.
///
/// A command takes the window it was invoked from and finds its handle here by its label: the handle itself is read on
/// the main thread as the window is built, which a command on an async worker is not.
#[derive(Default)]
pub struct WindowHandles {
  handles: RwLock<HashMap<String, NonZeroIsize>>,
}

impl WindowHandles {
  /// Records a built window's native handle; nothing on a platform no window host draws on.
  pub fn register<R: Runtime>(&self, window: &WebviewWindow<R>) {
    let Some(read) = Self::read_native_handle(window) else {
      return;
    };

    match read.ok().and_then(NonZeroIsize::new) {
      Some(handle) => {
        self.write().insert(window.label().to_string(), handle);
      }
      None => log::error!(
        "Window '{}' has no native handle, so it cannot host a native viewport",
        window.label()
      ),
    }
  }

  /// Forgets a destroyed window, so no viewport is attached to a handle that names nothing.
  pub fn forget(&self, label: &str) {
    self.write().remove(label);
  }

  /// The native handle of the window labelled `label`, none where no such window was built or it is gone.
  pub fn get(&self, label: &str) -> Option<NonZeroIsize> {
    self.read().get(label).copied()
  }

  /// The handles, whatever a panicking writer left: an entry is only ever added or removed whole.
  fn read(&self) -> RwLockReadGuard<'_, HashMap<String, NonZeroIsize>> {
    self.handles.read().unwrap_or_else(PoisonError::into_inner)
  }

  fn write(&self) -> RwLockWriteGuard<'_, HashMap<String, NonZeroIsize>> {
    self.handles.write().unwrap_or_else(PoisonError::into_inner)
  }

  #[cfg(windows)]
  fn read_native_handle<R: Runtime>(window: &WebviewWindow<R>) -> Option<tauri::Result<isize>> {
    Some(window.hwnd().map(|hwnd| hwnd.0 as isize))
  }

  /// The window itself, not its content view, which wry replaces with its own after the window is built.
  #[cfg(target_os = "macos")]
  fn read_native_handle<R: Runtime>(window: &WebviewWindow<R>) -> Option<tauri::Result<isize>> {
    Some(window.ns_window().map(|ns_window| ns_window as isize))
  }

  #[cfg(not(any(windows, target_os = "macos")))]
  fn read_native_handle<R: Runtime>(_window: &WebviewWindow<R>) -> Option<tauri::Result<isize>> {
    None
  }
}
