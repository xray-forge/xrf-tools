use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::TextureState;

/// Close browse state and invalidate held or unfinished comparisons.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close"))]
#[tauri::command(rename = "close")]
pub fn textures_close(session_ids: Vec<SessionId>, state: State<'_, TextureState>) -> TauriResult {
  state.close(&session_ids)
}
