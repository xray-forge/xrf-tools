use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::scene::{SceneBinaryToolsDescription, SceneBinaryToolsPart, SelectedScenes};

/// Describe the open scene's AI map, detail objects and wall marks by their counts, none of their payload; read again
/// where a commit let them go.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_scene_binary_tools")]
pub(crate) async fn levels_describe_scene_binary_tools(
  session_id: SessionId,
  scenes: State<'_, SelectedScenes>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<SceneBinaryToolsDescription>>> {
  scenes
    .read_on_pool(
      &execution,
      session_id,
      "Describing the scene's binary tools",
      |current| Ok(SceneBinaryToolsPart::get(current)),
    )
    .await
}
