use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::level::{SelectedLevel, SelectedLevelDescription, SelectedLevels};

/// Restore the level `tool` holds in the asking window, the newest it opened, without reading the level again: as a
/// page attaches to it, or reads it again after a commit derived it anew. Its parts are read by their own queries.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_level")]
pub async fn levels_restore_level(
  levels: State<'_, SelectedLevels>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<SelectedLevelDescription>> {
  Ok(
    levels
      .restore_as(&SessionScope::of(&window, tool), SelectedLevel::describe)
      .into(),
  )
}
