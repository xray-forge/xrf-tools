use tauri::State;
use xrf_app_core::session::SessionId;

use crate::level::SelectedLevels;

/// Close the levels `session_ids` name, each once, unbinding it from its scope.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_level")]
pub fn levels_close_level(levels: State<'_, SelectedLevels>, session_ids: Vec<SessionId>) {
  levels.close(&session_ids);
}
