use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::scene::{SceneDrawDescription, SceneDrawPart, SelectedScenes};

/// Describe what the page needs of the open scene's picture beside it: the extent of the scene objects drawn, to frame
/// a camera on, and each scene object not drawn, its library object missing or unreadable; read again where a commit
/// let the library objects go. What the renderer could not compile or texture is the viewport's to answer.
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
