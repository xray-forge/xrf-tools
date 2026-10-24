use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::{LevelStart, LevelStartPart};

/// Describe where the open level opens, or none for a viewer to decide; decided again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_start"))]
#[tauri::command(rename = "describe_start")]
pub async fn levels_describe_start(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Option<LevelStart>>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Describing where the level starts",
    |current, search| Ok(LevelStartPart::get(current, search)),
  )
  .await
}
