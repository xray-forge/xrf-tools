use tauri::State;
use xrf_app_core::session::{SessionScope, SessionScopes};
use xrf_app_core::types::TauriResult;

use crate::SpawnSessionDescriptor;
use crate::state::SpawnFileState;

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
