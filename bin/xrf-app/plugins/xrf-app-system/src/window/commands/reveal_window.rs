use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::window::{InvokingWindow, WindowReveals};

/// Show the calling window once its page has drawn its first screen; a window already shown is left as it is.
// Asynchronous to run off the main thread: a synchronous command runs inside the webview's message callback, and
// showing the window from there never returns on Windows.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "reveal_window")]
pub(crate) async fn system_reveal_window(reveals: State<'_, WindowReveals>, window: InvokingWindow) -> TauriResult {
  reveals.reveal(window.get_label());

  Ok(())
}
