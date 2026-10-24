use std::sync::Arc;

use tauri::State;

use crate::core::session::{SessionRestore, SessionScope, SessionScopes, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel, SelectedLevelDescription};

/// Restore the level a tool holds, the newest it opened, without reading the level again: as a page attaches to it, or
/// reads it again after a commit derived it anew. Its parts are read by their own queries.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_level"))]
#[tauri::command(rename = "get_level")]
pub fn levels_get_level(
  state: State<'_, LevelState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<SelectedLevelDescription>> {
  let current: Option<Arc<SessionSnapshot<SelectedLevel>>> = state.levels.restore(&scopes.list(&scope))?;

  Ok(SessionRestore::from(
    current.map(|current| current.answer(current.describe())),
  ))
}
