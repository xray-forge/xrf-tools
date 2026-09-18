use tauri::State;

use crate::core::session::SessionRestore;
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel, SelectedLevelDescription};

/// Restore the committed level descriptor without reading the level again.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_level"))]
#[tauri::command(rename = "get_level")]
pub async fn levels_get_level(state: State<'_, LevelState>) -> TauriResult<SessionRestore<SelectedLevelDescription>> {
  Ok(SessionRestore::from(
    state.selected.get()?.map(|opened| opened.map(SelectedLevel::describe)),
  ))
}
