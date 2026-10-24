use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::VisualState;

/// Release only the openings owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_browse"))]
#[tauri::command(rename = "close_browse")]
pub async fn visuals_close_browse(session_ids: Vec<SessionId>, state: State<'_, VisualState>) -> TauriResult {
  state.browses.close(&session_ids)
}
