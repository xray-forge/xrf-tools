use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::console::LevelConsoleDefaults;
use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::LevelConsolePart;

/// Describe what the open level's game runs its console with of how levels are lit and exposed: its `user.ltx` over
/// its shipped defaults; read again where a commit let the shipped ones go.
#[cfg_attr(
  feature = "typescript-bindings",
  specta::specta(rename = "describe_console_defaults")
)]
#[tauri::command(rename = "describe_console_defaults")]
pub async fn levels_describe_console_defaults(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<LevelConsoleDefaults>>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Reading the game's console defaults",
    LevelConsolePart::get,
  )
  .await
}
