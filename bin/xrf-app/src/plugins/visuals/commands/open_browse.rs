use std::sync::Arc;

use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Remember the browsed roots with an identity independent from the selected model.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_browse"))]
#[tauri::command(rename = "open_browse")]
pub async fn visuals_open_browse(
  session_id: DocumentSessionId,
  roots: XrayRoots,
  state: State<'_, VisualState>,
) -> TauriResult<Arc<DocumentSnapshot<XrayRoots>>> {
  state.browsed.begin_open(session_id)?;
  state.browsed.commit_open(session_id, roots)
}
