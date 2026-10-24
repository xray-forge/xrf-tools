use tauri::State;
use xrf_app_core::session::SessionId;

use crate::VisualPoses;

/// Close the posed motions `session_ids` name, committed or opening.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_motion"))]
#[tauri::command(rename = "close_motion")]
pub fn visuals_close_motion(poses: State<'_, VisualPoses>, session_ids: Vec<SessionId>) {
  poses.close(&session_ids);
}
