use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;

use crate::parts::LevelMapPart;
use crate::state::{LevelMapDescription, LevelState, SelectedLevel};

/// Describe where the open level's map lies over it, as its `level.ltx` places it, or none where it places none; read
/// again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_level_map"))]
#[tauri::command(rename = "describe_level_map")]
pub async fn levels_describe_level_map(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Option<LevelMapDescription>>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Describing the level's map", move || {
      TauriResult::Ok(current.answer(LevelMapPart::get(&current, &assets.search(&current.roots)?)))
    })
    .await?
}
