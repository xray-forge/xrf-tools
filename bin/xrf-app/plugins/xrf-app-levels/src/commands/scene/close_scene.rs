use tauri::State;
use xrf_app_core::session::SessionId;

use crate::scene::SelectedScenes;

/// Close the scenes `session_ids` name, each once, unbinding it from its scope.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_scene")]
pub(crate) fn levels_close_scene(scenes: State<'_, SelectedScenes>, session_ids: Vec<SessionId>) {
  scenes.close(&session_ids);
}
