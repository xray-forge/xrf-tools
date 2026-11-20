use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::scene::{SelectedScene, SelectedSceneDescription, SelectedScenes};

/// Restore the newest scene `tool` opened in the asking window without reading it again, for a page attaching to it or
/// refreshing after a commit. Its parts have their own queries.
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
