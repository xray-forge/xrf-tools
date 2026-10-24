use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::{LevelMapDescription, LevelMapPart};

/// Describe where the open level's map lies over it, as its `level.ltx` places it, or none where it places none; read
/// again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_level_map"))]
#[tauri::command(rename = "describe_level_map")]
pub async fn levels_describe_level_map(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Option<LevelMapDescription>>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Describing the level's map",
    |current, search| Ok(LevelMapPart::get(current, search)),
  )
  .await
}
