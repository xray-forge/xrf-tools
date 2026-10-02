use tauri::State;

use crate::core::types::TauriResult;
use crate::core::window::{WebviewOptions, WebviewOptionsState};
use crate::plugins::system::webview::WebviewOptionsStatus;

/// Keep browser options for the next start, which is when the webview's browser takes new ones: written to the disk
/// off the window's thread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_webview_options"))]
#[tauri::command(rename = "set_webview_options")]
pub async fn system_set_webview_options(
  state: State<'_, WebviewOptionsState>,
  options: WebviewOptions,
) -> TauriResult<WebviewOptionsStatus> {
  state.choose(options)?;

  WebviewOptionsStatus::read(&state)
}
