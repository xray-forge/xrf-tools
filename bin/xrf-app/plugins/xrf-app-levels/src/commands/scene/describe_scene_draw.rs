use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::scene::{SceneDrawDescription, SceneDrawPart, SelectedScenes};

/// Describe the open scene's drawing: the extent of the drawn objects, to frame a camera on, and each object not drawn
/// because its library object is missing or unreadable.
///
/// Call again after a commit that invalidates the library objects. Compile and texture failures are reported by the
/// viewport, not here.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_scene_draw")]
pub(crate) async fn levels_describe_scene_draw(
  session_id: SessionId,
  scenes: State<'_, SelectedScenes>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<SceneDrawDescription>>> {
  let assets: AssetMountState = AssetMountState::clone(&assets);

  scenes
    .read_on_pool(
      &execution,
      session_id,
      "Describing the scene's drawing",
      move |current| {
        Ok(Arc::clone(
          &SceneDrawPart::get(current, &assets.search(&current.objects_roots)?).description,
        ))
      },
    )
    .await
}
