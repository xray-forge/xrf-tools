use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::window::{InvokingWindow, WindowReveals};

/// Put the calling window on screen: its page has something themed to show. A page loaded over a window already shown
/// changes nothing.
///
/// Asynchronous so it runs off the main thread: a synchronous command runs inside the webview's message callback there,
/// and showing the window from inside it never returns on Windows.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "reveal_window")]
pub async fn system_reveal_window(reveals: State<'_, WindowReveals>, window: InvokingWindow) -> TauriResult {
  reveals.reveal(window.get_label());

  Ok(())
}
