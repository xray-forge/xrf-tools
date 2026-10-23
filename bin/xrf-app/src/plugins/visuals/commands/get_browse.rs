use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::session::{SessionRestore, SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Restore the roots a tool browses, the newest it opened.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_browse"))]
#[tauri::command(rename = "get_browse")]
pub async fn visuals_get_browse(
  state: State<'_, VisualState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<XrayRoots>> {
  Ok(SessionRestore::from(state.browses.restore(&scopes.list(&scope))?))
}
