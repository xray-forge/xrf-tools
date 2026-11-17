use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_spawn_builder::{SpawnListedLevel, SpawnRootGamedata};
use xrf_utils::error_to_string;

/// The levels a spawn can be built from: every level `gamedata`'s `configs/game_levels.ltx` lists, in its order, each
/// marked by whether its directory under `levels_root` holds `level.ai`, `level.spawn` and `level.game`.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "list_levels")]
pub(crate) async fn spawn_list_levels(
  execution: State<'_, ExecutionState>,
  gamedata: PathBuf,
  levels_root: PathBuf,
) -> TauriResult<Vec<SpawnListedLevel>> {
  execution
    .run_blocking("Listing spawn levels", move || {
      SpawnRootGamedata::list_levels(&gamedata, &levels_root).map_err(error_to_string)
    })
    .await?
}
