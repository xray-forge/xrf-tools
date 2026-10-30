use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::LevelSpawnPart;
use crate::spawn::LevelSpawnObjectDetails;

/// Describe one of the open level's spawned objects, by its index among them, as a level object's spawn id names it.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_spawn_object")]
pub async fn levels_describe_spawn_object(
  session_id: SessionId,
  index: u32,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelSpawnObjectDetails>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Describing a level spawned object",
    move |current, search| {
      let spawn = LevelSpawnPart::get(current, search)?;

      spawn
        .objects
        .get(index as usize)
        .map(LevelSpawnObjectDetails::of)
        .ok_or_else(|| format!("The level spawns no object {index}"))
    },
  )
  .await
}
