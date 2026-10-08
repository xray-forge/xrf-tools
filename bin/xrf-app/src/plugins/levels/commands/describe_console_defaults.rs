use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::console_defaults::read_level_console;
use crate::plugins::levels::state::{LevelConsoleDefaults, LevelState, SelectedLevel};

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
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: LevelConsoleDefaults = execution
    .run_blocking("Reading the game's console defaults", move || {
      assets.with_probe(&current.roots, |probe| {
        read_level_console(probe, &current.roots).map_err(|error| error.to_string())
      })
    })
    .await???;

  Ok(SessionSnapshot { session_id, value })
}
