use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::VisualState;

/// Release only the openings owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_model"))]
#[tauri::command(rename = "close_model")]
pub async fn visuals_close_model(session_ids: Vec<SessionId>, state: State<'_, VisualState>) -> TauriResult {
  state.models.close(&session_ids)
}
