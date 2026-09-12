use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::session::SessionRestore;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Restore the committed browse scope.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_browse"))]
#[tauri::command(rename = "get_browse")]
pub async fn visuals_get_browse(state: State<'_, VisualState>) -> TauriResult<SessionRestore<XrayRoots>> {
  Ok(SessionRestore::from(state.browsed.get()?))
}
