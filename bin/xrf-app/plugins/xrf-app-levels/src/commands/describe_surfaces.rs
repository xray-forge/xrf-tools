use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_material::XraySurfaceDescriptor;

use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::LevelSurfacesPart;

/// Describe how the renderer draws each entry of the open level's shader table, in its order, so the page names a
/// surface by the shader id a packed surface carries; resolved again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_surfaces"))]
#[tauri::command(rename = "describe_surfaces")]
pub async fn levels_describe_surfaces(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<Vec<XraySurfaceDescriptor>>>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Describing the level's surfaces",
    |current, search| Ok(LevelSurfacesPart::get(current, search)),
  )
  .await
}
