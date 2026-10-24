use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::SpawnFileState;

/// Close the committed file and prevent unfinished opens from restoring it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_file"))]
#[tauri::command(rename = "close_file")]
pub fn spawn_close_file(session_ids: Vec<SessionId>, state: State<'_, SpawnFileState>) -> TauriResult {
  state.files.close(&session_ids)
}
