use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_math::Sphere;

use crate::scene::{SceneObjectId, SceneObjectSphere, SelectedScenes};

/// Locate one of the open scene's objects for a camera to frame, as a sphere in renderer space about its placed box or
/// its helper; none for an object that draws nothing.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "locate_scene_object")]
pub(crate) async fn levels_locate_scene_object(
  session_id: SessionId,
  object: SceneObjectId,
  scenes: State<'_, SelectedScenes>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Option<Sphere>>> {
  let assets: AssetMountState = AssetMountState::clone(&assets);

  scenes
    .read_on_pool(&execution, session_id, "Locating a scene object", move |current| {
      SceneObjectSphere::locate(current, &assets, &object)
    })
    .await
}
