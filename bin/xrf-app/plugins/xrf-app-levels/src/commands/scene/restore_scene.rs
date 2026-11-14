use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::scene::{SelectedScene, SelectedSceneDescription, SelectedScenes};

/// Restore the scene `tool` holds in the asking window, the newest it opened, without reading the scene again: as a
/// page attaches to it, or reads it again after a commit derived it anew. Its parts are read by their own queries.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_scene")]
pub(crate) async fn levels_restore_scene(
  scenes: State<'_, SelectedScenes>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<SelectedSceneDescription>> {
  Ok(
    scenes
      .restore_as(&SessionScope::of(&window, tool), SelectedScene::describe)
      .into(),
  )
}
