use std::collections::HashMap;
use std::num::NonZeroIsize;
use std::sync::RwLock;

use tauri::Runtime;
use tauri::webview::WebviewWindow;

/// The native handle of every window that can host a native viewport, by its label, recorded as each is built: its HWND
/// on Windows, its `NSWindow` on macOS, which the render plugin's window hosts draw through.
///
/// Commands are not generic over the runtime, so a command cannot take the calling window itself; it names the window
/// by label and finds its handle here.
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
        self
          .handles
          .write()
          .unwrap_or_else(|poisoned| poisoned.into_inner())
          .insert(window.label().to_string(), handle);
      }
      None => log::error!(
        "Window '{}' has no native handle, so it cannot host a native viewport",
        window.label()
      ),
    }
  }

  /// Forgets a destroyed window, so no viewport is attached to a handle that names nothing.
  pub fn forget(&self, label: &str) {
    self
      .handles
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .remove(label);
  }

  pub fn get(&self, label: &str) -> Option<NonZeroIsize> {
    self
      .handles
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .get(label)
      .copied()
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
