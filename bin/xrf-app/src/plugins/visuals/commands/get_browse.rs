use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::session::DocumentRestore;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Restore the committed browse scope.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_browse"))]
#[tauri::command(rename = "get_browse")]
pub async fn visuals_get_browse(state: State<'_, VisualState>) -> TauriResult<DocumentRestore<XrayRoots>> {
  Ok(DocumentRestore::from(state.browsed.get()?))
}
