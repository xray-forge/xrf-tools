use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Release only the openings owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_browse"))]
#[tauri::command(rename = "close_browse")]
pub async fn visuals_close_browse(session_ids: Vec<SessionId>, state: State<'_, VisualState>) -> TauriResult {
  state.browsed.close(&session_ids)
}
