use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::LevelState;

/// Release only the openings owned by the departing viewer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_level"))]
#[tauri::command(rename = "close_level")]
pub fn levels_close_level(session_ids: Vec<SessionId>, state: State<'_, LevelState>) -> TauriResult {
  state.levels.close(&session_ids)
}
