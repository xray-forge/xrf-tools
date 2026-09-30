use tauri::State;

use crate::core::types::TauriResult;
use crate::core::window::WebviewOptionsState;
use crate::plugins::system::webview::WebviewOptionsStatus;

/// Report which browser options the webview runs with, and which the next start applies.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_webview_options"))]
#[tauri::command(rename = "get_webview_options")]
pub fn system_get_webview_options(state: State<'_, WebviewOptionsState>) -> TauriResult<WebviewOptionsStatus> {
  WebviewOptionsStatus::read(&state)
}
