use tauri::State;
use xrf_app_core::logging::UiLog;

use crate::ui_log::UiLogLevel;

/// Report the most verbose level at which the backend can write any of the webview's records, whatever its scope, none
/// where it writes none, so a record below it is dropped before it is sent; the backend filters each record it is sent
/// again.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "get_ui_log_level")]
pub(crate) fn system_get_ui_log_level(log: State<'_, UiLog>) -> Option<UiLogLevel> {
  log.get_level().map(UiLogLevel::from)
}
