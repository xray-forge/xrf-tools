use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionRestore, SessionScope, SessionScopes, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel, SelectedLevelDescription};

/// Restore the level a tool holds, the newest it opened, without reading the level again: as a page attaches to it, or
/// reads it again after a commit; only where it starts is decided again, where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_level"))]
#[tauri::command(rename = "get_level")]
pub async fn levels_get_level(
  state: State<'_, LevelState>,
  scopes: State<'_, SessionScopes>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<SelectedLevelDescription>> {
  let Some(current): Option<Arc<SessionSnapshot<SelectedLevel>>> = state.levels.restore(&scopes.list(&scope))? else {
    return Ok(SessionRestore(None));
  };
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let described: SessionSnapshot<SelectedLevelDescription> = execution
    .run_blocking("Describing the level", move || {
      TauriResult::Ok(SelectedLevel::describe_at(&current, &assets.search(&current.roots)?))
    })
    .await??;

  Ok(SessionRestore::from(Some(described)))
}
