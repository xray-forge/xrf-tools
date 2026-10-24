use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;

use crate::state::{LevelConsoleDefaults, LevelState, SelectedLevel};

/// Describe what the open level's game runs its console with of how levels are lit and exposed: its `user.ltx` over
/// its shipped defaults.
#[cfg_attr(
  feature = "typescript-bindings",
  specta::specta(rename = "describe_console_defaults")
)]
#[tauri::command(rename = "describe_console_defaults")]
pub async fn levels_describe_console_defaults(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelConsoleDefaults>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: SessionSnapshot<LevelConsoleDefaults> = execution
    .run_blocking("Reading the game's console defaults", move || {
      LevelConsoleDefaults::read(&assets.search(&current.roots)?, &current.roots)
        .map(|defaults| current.answer(defaults))
        .map_err(|error| error.to_string())
    })
    .await??;

  Ok(value)
}
