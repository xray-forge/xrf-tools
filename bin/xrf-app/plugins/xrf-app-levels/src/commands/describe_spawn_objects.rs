use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::spawn::LevelSpawnObjectsDescription;

/// Describe the open level's spawned objects the viewer draws, and the visuals they stand as, reading no visual; an
/// error where the spawn cannot be read.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_spawn_objects")]
pub async fn levels_describe_spawn_objects(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelSpawnObjectsDescription>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Describing the level spawned objects",
    LevelSpawnObjectsDescription::read,
  )
  .await
}
