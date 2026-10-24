use tauri::State;
use xrf_app_core::session::SessionId;

use crate::SpawnFiles;

/// Close the spawns `session_ids` name, committed or opening; a closed one answers no read.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_file"))]
#[tauri::command(rename = "close_file")]
pub fn spawn_close_file(files: State<'_, SpawnFiles>, session_ids: Vec<SessionId>) {
  files.close(&session_ids);
}
