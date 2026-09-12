use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Release only the openings owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_model"))]
#[tauri::command(rename = "close_model")]
pub async fn visuals_close_model(session_ids: Vec<SessionId>, state: State<'_, VisualState>) -> TauriResult {
  state.selected.close(&session_ids)
}
