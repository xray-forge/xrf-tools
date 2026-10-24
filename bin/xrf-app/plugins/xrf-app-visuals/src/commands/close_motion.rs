use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::VisualState;

/// Release only the posed motions owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_motion"))]
#[tauri::command(rename = "close_motion")]
pub async fn visuals_close_motion(session_ids: Vec<SessionId>, state: State<'_, VisualState>) -> TauriResult {
  state.poses.close(&session_ids)
}
