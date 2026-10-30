use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::objects::LevelObjectsDescription;

/// Describe the open level's objects and the visuals they are drawn as, reading no visual; an error where the spawn
/// cannot be read.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_objects")]
pub async fn levels_describe_objects(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelObjectsDescription>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Describing the level objects",
    LevelObjectsDescription::read,
  )
  .await
}
