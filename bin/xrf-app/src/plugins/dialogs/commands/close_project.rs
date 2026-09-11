use tauri::State;

use crate::core::session::DocumentSessionId;
use crate::core::types::TauriResult;
use crate::plugins::dialogs::state::DialogProjectState;

/// Releases only the committed and pending openings owned by the closing frontend.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub async fn dialogs_close_project(
  session_ids: Vec<DocumentSessionId>,
  state: State<'_, DialogProjectState>,
) -> TauriResult {
  state.close(&session_ids)
}
