use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::scene::{SceneObjectsDescription, SceneObjectsPart, SelectedScenes};

/// Describe the objects of the open scene's object tools, resolving each library object in the scene's objects roots.
///
/// Call again after a commit that invalidates them.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_scene_objects")]
pub(crate) async fn levels_describe_scene_objects(
  session_id: SessionId,
  scenes: State<'_, SelectedScenes>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<SceneObjectsDescription>>> {
  let assets: AssetMountState = AssetMountState::clone(&assets);

  scenes
    .read_on_pool(
      &execution,
      session_id,
      "Describing the scene's objects",
      move |current| Ok(SceneObjectsPart::get(current, &assets.search(&current.objects_roots)?)),
    )
    .await
}
