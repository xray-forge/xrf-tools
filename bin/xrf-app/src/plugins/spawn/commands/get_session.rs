use tauri::State;

use crate::core::session::{SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionDescriptor;
use crate::plugins::spawn::state::SpawnFileState;

/// Restore the spawn a tool holds, the newest it opened, without cloning any of the large chunks.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_session"))]
#[tauri::command(rename = "get_session")]
pub fn spawn_get_session(
  state: State<'_, SpawnFileState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<Option<SpawnSessionDescriptor>> {
  state.restore_descriptor(&scopes.list(&scope))
}
