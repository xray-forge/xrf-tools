use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::scene::{SceneHelpersDescription, SceneHelpersPart, SelectedScenes};

/// Describe the open scene's helpers beside their picture: where they reach, to frame a camera on a scene drawing them
/// alone, and how many each helped tool draws; read again where a commit let them go.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_scene_helpers")]
pub(crate) async fn levels_describe_scene_helpers(
  session_id: SessionId,
  scenes: State<'_, SelectedScenes>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<SceneHelpersDescription>>> {
  scenes
    .read_on_pool(&execution, session_id, "Describing the scene's helpers", |current| {
      Ok(Arc::clone(&SceneHelpersPart::get(current).description))
    })
    .await
}
