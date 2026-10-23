use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionRestore, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::level_read::describe_level;
use crate::plugins::levels::state::{LevelState, SelectedLevel, SelectedLevelDescription};

/// Restore the committed level descriptor without reading the level again; only where it starts is decided again,
/// where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_level"))]
#[tauri::command(rename = "get_level")]
pub async fn levels_get_level(
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionRestore<SelectedLevelDescription>> {
  let Some(current): Option<Arc<SessionSnapshot<SelectedLevel>>> = state.selected.get()? else {
    return Ok(SessionRestore(None));
  };
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let described: SessionSnapshot<SelectedLevelDescription> = execution
    .run_blocking("Describing the level", move || {
      TauriResult::Ok(describe_level(&current, &assets.search(&current.roots)?))
    })
    .await??;

  Ok(SessionRestore::from(Some(described)))
}
