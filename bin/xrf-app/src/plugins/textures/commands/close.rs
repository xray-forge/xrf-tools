use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::textures::state::TextureState;

/// Close browse state and invalidate held or unfinished comparisons.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close"))]
#[tauri::command(rename = "close")]
pub async fn textures_close(session_ids: Vec<SessionId>, state: State<'_, TextureState>) -> TauriResult {
  state.close(&session_ids)
}
