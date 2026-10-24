use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Release only the posed motions owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_motion"))]
#[tauri::command(rename = "close_motion")]
pub async fn visuals_close_motion(session_ids: Vec<SessionId>, state: State<'_, VisualState>) -> TauriResult {
  state.poses.close(&session_ids)
}
