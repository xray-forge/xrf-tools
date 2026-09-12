use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::exports::state::ExportsProjectState;

/// Releases only the committed and pending openings owned by the closing frontend.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub async fn exports_close_project(session_ids: Vec<SessionId>, state: State<'_, ExportsProjectState>) -> TauriResult {
  state.close(&session_ids)
}
