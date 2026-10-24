use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;

use crate::parts::LevelStartPart;
use crate::state::{LevelStart, LevelState, SelectedLevel};

/// Describe where the open level opens, or none for a viewer to decide; decided again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_start"))]
#[tauri::command(rename = "describe_start")]
pub async fn levels_describe_start(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Option<LevelStart>>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Describing where the level starts", move || {
      TauriResult::Ok(current.answer(LevelStartPart::get(&current, &assets.search(&current.roots)?)))
    })
    .await?
}
