use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::levels::state::LevelState;

/// Release only the openings owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_level"))]
#[tauri::command(rename = "close_level")]
pub async fn levels_close_level(session_ids: Vec<SessionId>, state: State<'_, LevelState>) -> TauriResult {
  state.selected.close(&session_ids)
}
