use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::logging::UiLog;

use crate::ui_log::{UiLogBatch, UiLogRecord};

/// Write a batch of the webview's records to this run's UI log in their order, each filtered again as the backend
/// filters its own; a batch over 512 records is refused whole.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "write_ui_log")]
pub(crate) async fn system_write_ui_log(log: State<'_, UiLog>, records: Vec<UiLogRecord>) -> TauriResult {
  // Asynchronous so the writes run on the runtime's workers, off the thread the window's events run on.
  UiLogBatch::new(records)?.write_to(&log)
}
