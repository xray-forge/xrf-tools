use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_material::XraySurfaceDescriptor;

use crate::parts::LevelSurfacesPart;
use crate::state::{LevelState, SelectedLevel};

/// Describe how the renderer draws each entry of the open level's shader table, in its order, so the page names a
/// surface by the shader id a packed surface carries; resolved again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_surfaces"))]
#[tauri::command(rename = "describe_surfaces")]
pub async fn levels_describe_surfaces(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Vec<XraySurfaceDescriptor>>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Describing the level's surfaces", move || {
      let surfaces: Arc<Vec<XraySurfaceDescriptor>> = LevelSurfacesPart::get(&current, &assets.search(&current.roots)?);

      TauriResult::Ok(current.answer(surfaces.to_vec()))
    })
    .await?
}
